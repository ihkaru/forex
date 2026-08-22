use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::{PolaNConfig, Signal, SignalAction, SignalStatus, TfPairSpec};
use crate::ports::{MarketContext, StrategyPort};

use super::detector::{calculate_atr, calculate_ema, SwingPointDetector};
use super::formation::{PolaNFormationEngine, PolaNType};

/// Strategi Lengkap: Pola N Traders Family (Murni Decoupled dari Sumber Data)
#[derive(Debug, Clone)]
pub struct PolaNStrategy {
    pub name: String,
    pub swing_detector: SwingPointDetector,
    pub formation_engine: PolaNFormationEngine,
    pub ema_trend_period: usize,
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
            swing_detector: SwingPointDetector::new(5, 3),
            formation_engine: PolaNFormationEngine::default(),
            ema_trend_period: 50,
        }
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
            ema_trend_period: 50,
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
            ema_trend_period: 50,
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

        let dynamic_buffer = if let Some(atr) = calculate_atr(ctx.candles, 14) {
            (atr * dec!(0.35)).max(spec.pip_size * dec!(3.0))
        } else {
            spec.pip_size * dec!(3.0)
        };
        engine.pip_buffer = dynamic_buffer;

        let current_price = ctx.current_tick.bid;
        if let Some(formation) = engine.evaluate_swings(&swings, current_price) {
            let impulse = (formation.point_2 - formation.point_1).abs();
            let impulse_pips = spec.price_diff_to_pips(impulse);

            if impulse_pips < spec.min_sl_tp_pips {
                return Ok(None);
            }

            let ema_fast = calculate_ema(ctx.candles, 20);
            let ema_slow = calculate_ema(ctx.candles, self.ema_trend_period);

            if let (Some(fast), Some(slow)) = (ema_fast, ema_slow) {
                match formation.pattern_type {
                    PolaNType::BullishN => {
                        if fast < slow {
                            return Ok(None);
                        }
                    }
                    PolaNType::BearishN => {
                        if fast > slow {
                            return Ok(None);
                        }
                    }
                }
            }

            let (action, rationale) = match formation.pattern_type {
                PolaNType::BullishN => (
                    SignalAction::BuyLimit,
                    format!(
                        "Pola N Bullish Terkonfirmasi: L1 ({}) -> H1 ({}) -> Retest Higher Low L2 ({}) [Trend EMA Bullish]",
                        formation.point_1, formation.point_2, formation.point_3
                    ),
                ),
                PolaNType::BearishN => (
                    SignalAction::SellLimit,
                    format!(
                        "Pola N Bearish Terkonfirmasi: H1 ({}) -> L1 ({}) -> Retest Lower High H2 ({}) [Trend EMA Bearish]",
                        formation.point_1, formation.point_2, formation.point_3
                    ),
                ),
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

            return Ok(Some(signal));
        }

        Ok(None)
    }
}
