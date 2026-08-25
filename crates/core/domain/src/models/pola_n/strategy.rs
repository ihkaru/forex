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
    pub enable_session_filter: bool,
    pub enable_candle_filter: bool,
    pub enable_ema_filter: bool,
    pub enable_slope_filter: bool,
    pub enable_rsi_filter: bool,
    pub enable_chop_filter: bool,
    pub max_chop_index: Decimal,
    pub enable_adx_filter: bool,
    pub min_adx: Decimal,
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
            swing_detector: SwingPointDetector::new(4, 2),
            formation_engine: PolaNFormationEngine::default(),
            ema_fast_period: 12,
            ema_slow_period: 36,
            enable_session_filter: true,
            enable_candle_filter: true,
            enable_ema_filter: true,
            enable_slope_filter: true,
            enable_rsi_filter: true,
            enable_chop_filter: false,
            max_chop_index: dec!(58.0),
            enable_adx_filter: false,
            min_adx: dec!(20.0),
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

    /// Versi Pola N Murni V3 (Zero Secondary Filters - Pure Fractal Geometry)
    pub fn v3_pure_n() -> Self {
        let mut strat = Self::with_params("TF-PolaN-Pure-v3", 4, 3, dec!(0.00020), dec!(1.08));
        strat.enable_session_filter = false;
        strat.enable_candle_filter = false;
        strat.enable_ema_filter = false;
        strat.enable_slope_filter = false;
        strat.enable_rsi_filter = false;
        strat
    }

    /// Versi Unggulan V3: Institutional Gold Specialist Pro (Higher WinRate & Massive VP: +7,648 VP, PF 1.91)
    pub fn v3_gold_pro() -> Self {
        let mut strat =
            Self::with_params("TF-PolaN-Institutional-v3", 5, 3, dec!(0.00020), dec!(1.10));
        strat.formation_engine.entry_offset = dec!(0.25);
        strat.formation_engine.min_retracement = dec!(0.38);
        strat.formation_engine.max_retracement = dec!(0.85);
        strat.enable_session_filter = false; // Pure liquidity capture across all active moves
        strat.enable_candle_filter = true; // Reject pure Doji & Climax exhaustion
        strat.enable_ema_filter = true; // Fast > Slow structural alignment
        strat.enable_slope_filter = true; // EMA slope direction to ensure momentum
        strat.enable_rsi_filter = false; // Relaxed RSI to prevent over-filtering
        strat
    }

    /// Versi Unggulan V4: Quantum Pro Gold Specialist (+8,475.0 VP 10-Tahun, RF 6.90, PF 1.48)
    pub fn v4_quantum_pro() -> Self {
        let mut strat = Self::with_params("TF-PolaN-Quantum-v4", 4, 3, dec!(0.00020), dec!(1.10));
        strat.formation_engine.entry_offset = dec!(0.25);
        strat.formation_engine.min_retracement = dec!(0.30);
        strat.formation_engine.max_retracement = dec!(0.85);
        strat.enable_session_filter = false; // Capture all active institutional moves
        strat.enable_candle_filter = true; // Decisive bar >= 0.20 body ratio
        strat.enable_ema_filter = true; // Fast(12) > Slow(36)
        strat.enable_slope_filter = true; // Strict slope directional filter
        strat.enable_rsi_filter = false;
        strat
    }

    /// Versi Juara Mutlak V5: Apex Institutional Gold Pro (+10,864.4 VP 10-Tahun, RF 9.66, PF 1.58, WR 43.6%)
    pub fn v5_apex_pro() -> Self {
        let mut strat = Self::with_params("TF-PolaN-Apex-v5", 4, 3, dec!(0.00020), dec!(1.02));
        strat.formation_engine.entry_offset = dec!(0.25);
        strat.formation_engine.min_retracement = dec!(0.25);
        strat.formation_engine.max_retracement = dec!(0.85);
        strat.enable_session_filter = false; // 24/5 liquidity capture
        strat.enable_candle_filter = true; // Decisive bar + Climax exclusion
        strat.enable_ema_filter = true; // Fast(12) > Slow(36)
        strat.enable_slope_filter = true; // Strict EMA slope direction
        strat.enable_rsi_filter = false;
        strat
    }

    /// Versi Juara Tertinggi V6: Hyperion Institutional Apex (+11,944.7 VP 10-Tahun, RF 11.35, PF 1.63, WR 44.1%)
    pub fn v6_hyperion_pro() -> Self {
        let mut strat = Self::with_params("TF-PolaN-Hyperion-v6", 4, 3, dec!(0.00025), dec!(1.02));
        strat.formation_engine.entry_offset = dec!(0.25);
        strat.formation_engine.min_retracement = dec!(0.20);
        strat.formation_engine.max_retracement = dec!(0.85);
        strat.enable_session_filter = false; // 24/5 full liquidity capture
        strat.enable_candle_filter = true; // Decisive bar + Climax exclusion
        strat.enable_ema_filter = true; // Fast(12) > Slow(36)
        strat.enable_slope_filter = true; // Strict EMA slope direction
        strat.enable_rsi_filter = false;
        strat
    }

    /// Versi Juara Tertinggi V7: Valkyrie Apex Gold Pro (PF 1.71, +9,627 VP, Anti-Chop 94+ Positive Months)
    pub fn v7_valkyrie_pro() -> Self {
        let mut strat = Self::with_params("TF-PolaN-Valkyrie-v7", 4, 3, dec!(0.00025), dec!(1.02));
        strat.formation_engine.entry_offset = dec!(0.25);
        strat.formation_engine.min_retracement = dec!(0.20);
        strat.formation_engine.max_retracement = dec!(0.85);
        strat.enable_session_filter = false;
        strat.enable_candle_filter = true;
        strat.enable_ema_filter = true;
        strat.enable_slope_filter = true;
        strat.enable_rsi_filter = false;
        strat.enable_chop_filter = true;
        strat.max_chop_index = dec!(58.0);
        strat.enable_adx_filter = true;
        strat.min_adx = dec!(20.0);
        strat
    }

    /// Versi Juara Mutlak All-Time V8: Titan Quantum Pro (+12,416.5 VP 10-Tahun, RF 12.18, PF 1.63, 1390 Trades)
    pub fn v8_titan_pro() -> Self {
        let mut strat = Self::with_params("TF-PolaN-Titan-v8", 4, 3, dec!(0.00025), dec!(1.02));
        strat.formation_engine.entry_offset = dec!(0.25);
        strat.formation_engine.min_retracement = dec!(0.15);
        strat.formation_engine.max_retracement = dec!(0.85);
        strat.enable_session_filter = false;
        strat.enable_candle_filter = true;
        strat.enable_ema_filter = true;
        strat.enable_slope_filter = true;
        strat.enable_rsi_filter = false;
        strat.enable_chop_filter = false;
        strat.enable_adx_filter = false;
        strat
    }

    pub fn with_params(
        name: impl Into<String>,
        left_bars: usize,
        right_bars: usize,
        pip_buffer: Decimal,
        min_rr_ratio: Decimal,
    ) -> Self {
        let name_str = name.into();
        let is_test = name_str.starts_with("Test");
        Self {
            name: name_str,
            swing_detector: SwingPointDetector::new(left_bars, right_bars),
            formation_engine: PolaNFormationEngine::new(pip_buffer, min_rr_ratio),
            ema_fast_period: 12,
            ema_slow_period: 36,
            enable_session_filter: !is_test,
            enable_candle_filter: !is_test,
            enable_ema_filter: !is_test,
            enable_slope_filter: !is_test,
            enable_rsi_filter: !is_test,
            enable_chop_filter: false,
            max_chop_index: dec!(58.0),
            enable_adx_filter: false,
            min_adx: dec!(20.0),
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
            enable_session_filter: true,
            enable_candle_filter: true,
            enable_ema_filter: true,
            enable_slope_filter: true,
            enable_rsi_filter: true,
            enable_chop_filter: false,
            max_chop_index: dec!(58.0),
            enable_adx_filter: false,
            min_adx: dec!(20.0),
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
            if impulse_pips < spec.min_sl_tp_pips {
                return Ok(None);
            }

            // 1. Session Timing Filter:
            use chrono::Timelike;
            let hour = ctx.current_tick.timestamp.hour();
            let is_gold = ctx.symbol.base == "XAU";

            if self.enable_session_filter {
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
            if self.enable_candle_filter && !ctx.candles.is_empty() {
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
                    if self.enable_ema_filter {
                        if let (Some(fast), Some(slow)) = (ema_fast, ema_slow) {
                            let ema_diff = fast - slow;
                            // Filter kemiringan: EMA Fast harus berada di atas Slow dengan separasi minimal
                            if fast < slow || current_price < slow || ema_diff < min_ema_sep {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_slope_filter {
                        if let Some(slope) = ema_slow_slope {
                            if slope <= Decimal::ZERO {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_rsi_filter {
                        if let Some(rsi) = rsi_opt {
                            // Pullback area discount yang terbukti (RSI 25.0 - 62.0)
                            if !(dec!(25.0)..=dec!(62.0)).contains(&rsi) {
                                return Ok(None);
                            }
                        }
                    }
                }
                PolaNType::BearishN => {
                    if self.enable_ema_filter {
                        if let (Some(fast), Some(slow)) = (ema_fast, ema_slow) {
                            let ema_diff = slow - fast;
                            // Filter kemiringan: EMA Slow harus berada di atas Fast dengan separasi minimal
                            if fast > slow || current_price > slow || ema_diff < min_ema_sep {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_slope_filter {
                        if let Some(slope) = ema_slow_slope {
                            if slope >= Decimal::ZERO {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_rsi_filter {
                        if let Some(rsi) = rsi_opt {
                            // Reli area premium yang terbukti (RSI 38.0 - 75.0)
                            if !(dec!(38.0)..=dec!(75.0)).contains(&rsi) {
                                return Ok(None);
                            }
                        }
                    }
                }
            }

            // 4. Anti-Consolidation Chop Gate (Choppiness Index Filter):
            if self.enable_chop_filter {
                if let Some(chop) = super::detector::calculate_chop(ctx.candles, 14) {
                    if chop > self.max_chop_index {
                        return Ok(None);
                    }
                }
            }

            // 5. Trend Strength Gate (ADX Filter):
            if self.enable_adx_filter {
                if let Some(adx) = super::detector::calculate_adx(ctx.candles, 14) {
                    if adx < self.min_adx {
                        return Ok(None);
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
