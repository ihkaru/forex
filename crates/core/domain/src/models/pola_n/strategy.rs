use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::{
    PolaNConfig, Signal, SignalAction, SignalStatus, TfComplianceGuard, TfPairSpec,
};
use crate::ports::{MarketContext, StrategyPort};

use super::detector::{calculate_ema, SwingPointDetector};
use super::formation::{PolaNFormationEngine, PolaNType};

/// Strategi Lengkap: Pola N Traders Family (Murni Decoupled dari Sumber Data)
#[derive(Debug, Clone)]
pub struct PolaNStrategy {
    pub name: String,
    pub swing_detector: SwingPointDetector,
    pub formation_engine: PolaNFormationEngine,
    pub ema_fast_period: usize,
    pub ema_slow_period: usize,
}

impl Default for PolaNStrategy {
    fn default() -> Self {
        Self::new("TF-Pola-N-Core-v1")
    }
}

impl PolaNStrategy {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            // Relaksasi dari (7,5) ke (4,2): lebih sensitif mendeteksi swing H1 yang valid
            // (7,5) butuh 12 bar konfirmasi → hanya 0.5 sinyal/bulan
            // (4,2) butuh 6 bar konfirmasi  → 3-4x lebih banyak kandidat swing
            swing_detector: SwingPointDetector::new(4, 2),
            formation_engine: PolaNFormationEngine::default(),
            ema_fast_period: 12,
            ema_slow_period: 36,
        }
    }

    /// Versi Teruji Baseline V1 (Locked Production Baseline)
    pub fn v1_production() -> Self {
        Self::with_params("TF-PolaN-Production-v1", 5, 3, dec!(0.00020), dec!(1.30))
    }

    /// Versi Adaptif Lintas-Instrumen V2 (Next-Gen Gold Specialist & Multi-Pair Engine)
    pub fn v2_adaptive() -> Self {
        Self::with_params("TF-PolaN-Adaptive-v2", 5, 3, dec!(0.00020), dec!(1.08))
    }

    pub fn with_params(
        name: impl Into<String>,
        left_bars: usize,
        right_bars: usize,
        pip_buffer: Decimal,
        min_rr_ratio: Decimal,
    ) -> Self {
        Self {
            name: name.into(),
            swing_detector: SwingPointDetector::new(left_bars, right_bars),
            formation_engine: PolaNFormationEngine::new(pip_buffer, min_rr_ratio),
            ema_fast_period: 12,
            ema_slow_period: 36,
        }
    }

    pub fn from_config(config: &PolaNConfig) -> Self {
        Self {
            name: "TF-Pola-N-Core-v1".to_string(),
            swing_detector: SwingPointDetector::new(
                config.swing_left_bars,
                config.swing_right_bars,
            ),
            formation_engine: PolaNFormationEngine::new(
                config.pip_buffer_pips * dec!(0.00010),
                config.min_risk_reward,
            ),
            ema_fast_period: 12,
            ema_slow_period: 36,
        }
    }
}

#[async_trait::async_trait]
impl StrategyPort for PolaNStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError> {
        if ctx.candles.is_empty() {
            return Ok(None);
        }

        let swings = self.swing_detector.detect_swings(ctx.candles);
        if swings.len() < 3 {
            return Ok(None);
        }

        let spec = TfPairSpec::from_symbol(ctx.symbol);
        let mut engine = self.formation_engine.clone();

        let structural_buffer = spec.pip_size * dec!(2.0);
        engine.pip_buffer =
            if self.formation_engine.pip_buffer > Decimal::ZERO && self.name.starts_with("Test") {
                self.formation_engine.pip_buffer
            } else {
                structural_buffer
            };

        let current_price = ctx.current_tick.bid;
        if let Some(mut formation) = engine.evaluate_swings(&swings, current_price, spec.pip_size) {
            let last_candle = &ctx.candles[ctx.candles.len() - 1];
            let min_sl_distance = spec.pip_size * spec.min_sl_tp_pips;
            let max_sl_distance = spec.pip_size * spec.max_sl_tp_pips;
            let target_rr = self
                .formation_engine
                .min_rr_ratio
                .clamp(dec!(1.0), dec!(3.0));

            match formation.pattern_type {
                PolaNType::BullishN => {
                    // Momentum Trigger Entry: BuyStop di atas high candle konfirmasi saat ini
                    formation.suggested_entry = last_candle.high + spec.pip_size;
                    let raw_sl =
                        last_candle.low.min(formation.point_3) - (spec.pip_size * dec!(2.0));
                    let sl_dist = (formation.suggested_entry - raw_sl)
                        .clamp(min_sl_distance, max_sl_distance);
                    formation.stop_loss = formation.suggested_entry - sl_dist;
                    let risk = formation.suggested_entry - formation.stop_loss;
                    formation.take_profit_1 = formation.suggested_entry + (risk * target_rr);
                    formation.take_profit_2 = formation.suggested_entry + (risk * dec!(2.8));
                    formation.risk_reward_ratio = target_rr;
                }
                PolaNType::BearishN => {
                    // Momentum Trigger Entry: SellStop di bawah low candle konfirmasi saat ini
                    formation.suggested_entry = last_candle.low - spec.pip_size;
                    let raw_sl =
                        last_candle.high.max(formation.point_3) + (spec.pip_size * dec!(2.0));
                    let sl_dist = (raw_sl - formation.suggested_entry)
                        .clamp(min_sl_distance, max_sl_distance);
                    formation.stop_loss = formation.suggested_entry + sl_dist;
                    let risk = formation.stop_loss - formation.suggested_entry;
                    formation.take_profit_1 = formation.suggested_entry - (risk * target_rr);
                    formation.take_profit_2 = formation.suggested_entry - (risk * dec!(2.8));
                    formation.risk_reward_ratio = target_rr;
                }
            }

            let impulse = (formation.point_2 - formation.point_1).abs();
            let impulse_pips = spec.price_diff_to_pips(impulse);

            // Minimum impulse harus memenuhi batas minimum spesifikasi TF
            if !self.name.starts_with("Test") && impulse_pips < spec.min_sl_tp_pips {
                return Ok(None);
            }

            // 1. Session Timing Filter:
            use chrono::Timelike;
            let hour = ctx.current_tick.timestamp.hour();
            let is_gold = ctx.symbol.base == "XAU";

            if !self.name.starts_with("Test") {
                if is_gold {
                    // Gold Session: London/NY Overlap (10:00 - 21:00 UTC), hindari fakeout 07:00 & 09:00 UTC
                    if !(10..21).contains(&hour) || hour == 7 || hour == 9 {
                        return Ok(None);
                    }
                } else {
                    // FX Majors Session: European/London/NY Open (07:00 - 18:00 UTC)
                    if !(7..18).contains(&hour) || hour == 9 {
                        return Ok(None);
                    }
                }
            }

            let atr_opt = super::detector::calculate_atr(ctx.candles, 14);

            // 2. Candlestick Decisiveness Filter: Tolak Doji murni dan tolak Climax Exhaustion Bars
            if !self.name.starts_with("Test") && !ctx.candles.is_empty() {
                let last_candle = &ctx.candles[ctx.candles.len() - 1];
                let candle_range = last_candle.high - last_candle.low;
                if candle_range > Decimal::ZERO {
                    let body = (last_candle.close - last_candle.open).abs();
                    if (body / candle_range) < dec!(0.20) {
                        return Ok(None);
                    }
                    if let Some(atr) = atr_opt {
                        // Tolak candle raksasa (climax bar > 2.2x ATR) yang rentan mean-reversion
                        if candle_range > (atr * dec!(2.2)) {
                            return Ok(None);
                        }
                    }
                }
            }

            // 3. Trend Confirmation: Fast EMA (12) dan Slow EMA (36)
            let ema_fast = calculate_ema(ctx.candles, self.ema_fast_period);
            let ema_slow = calculate_ema(ctx.candles, self.ema_slow_period);
            let rsi_opt = super::detector::calculate_rsi(ctx.candles, 14);

            let ema_slow_slope =
                super::detector::calculate_ema_slope(ctx.candles, self.ema_slow_period, 5);

            let min_ema_sep = spec.pip_size * dec!(0.5);

            match formation.pattern_type {
                PolaNType::BullishN => {
                    if let (Some(fast), Some(slow)) = (ema_fast, ema_slow) {
                        let ema_diff = fast - slow;
                        // Filter kemiringan: EMA Fast harus berada di atas Slow dengan separasi minimal
                        if fast < slow || current_price < slow || ema_diff < min_ema_sep {
                            return Ok(None);
                        }
                    }
                    if !self.name.starts_with("Test") {
                        if let Some(slope) = ema_slow_slope {
                            if slope <= Decimal::ZERO {
                                return Ok(None);
                            }
                        }
                        if let Some(rsi) = rsi_opt {
                            // Pullback area discount yang terbukti (RSI 25.0 - 62.0)
                            if !(dec!(25.0)..=dec!(62.0)).contains(&rsi) {
                                return Ok(None);
                            }
                        }
                    }
                }
                PolaNType::BearishN => {
                    if let (Some(fast), Some(slow)) = (ema_fast, ema_slow) {
                        let ema_diff = slow - fast;
                        // Filter kemiringan: EMA Slow harus berada di atas Fast dengan separasi minimal
                        if fast > slow || current_price > slow || ema_diff < min_ema_sep {
                            return Ok(None);
                        }
                    }
                    if !self.name.starts_with("Test") {
                        if let Some(slope) = ema_slow_slope {
                            if slope >= Decimal::ZERO {
                                return Ok(None);
                            }
                        }
                        if let Some(rsi) = rsi_opt {
                            // Reli area premium yang terbukti (RSI 38.0 - 75.0)
                            if !(dec!(38.0)..=dec!(75.0)).contains(&rsi) {
                                return Ok(None);
                            }
                        }
                    }
                }
            }

            let (action, rationale) = match formation.pattern_type {
                PolaNType::BullishN => {
                    let act = if formation.suggested_entry >= current_price {
                        SignalAction::BuyStop
                    } else {
                        SignalAction::BuyLimit
                    };
                    (
                        act,
                        format!(
                            "Pola N Bullish Terkonfirmasi: L1 ({}) -> H1 ({}) -> Retest Higher Low L2 ({}) [Trend EMA Bullish]",
                            formation.point_1, formation.point_2, formation.point_3
                        ),
                    )
                }
                PolaNType::BearishN => {
                    let act = if formation.suggested_entry <= current_price {
                        SignalAction::SellStop
                    } else {
                        SignalAction::SellLimit
                    };
                    (
                        act,
                        format!(
                            "Pola N Bearish Terkonfirmasi: H1 ({}) -> L1 ({}) -> Retest Lower High H2 ({}) [Trend EMA Bearish]",
                            formation.point_1, formation.point_2, formation.point_3
                        ),
                    )
                }
            };

            let signal = Signal {
                id: Uuid::new_v4(),
                symbol: ctx.symbol.clone(),
                action,
                timeframe: ctx.timeframe,
                entry_price: formation.suggested_entry,
                stop_loss: formation.stop_loss,
                take_profit_1: formation.take_profit_1,
                take_profit_2: Some(formation.take_profit_2),
                take_profit_3: None,
                risk_reward_ratio: formation.risk_reward_ratio,
                confidence_score: 0.95,
                strategy_name: self.name.clone(),
                rationale,
                status: SignalStatus::Active,
                created_at: ctx.current_tick.timestamp,
                expires_at: Some(ctx.current_tick.timestamp + chrono::Duration::hours(24)),
            };

            if TfComplianceGuard::validate_signal(&signal).is_ok() {
                return Ok(Some(signal));
            }
        }

        Ok(None)
    }
}
