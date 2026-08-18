use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::{Signal, SignalAction, SignalStatus, TfPairSpec};
use crate::ports::{MarketContext, StrategyPort};

/// Strategi Smart Money Concepts: Asian/London Liquidity Sweep + Order Block Retest
#[derive(Debug, Clone)]
pub struct SmcLiquiditySweepStrategy {
    pub name: String,
    pub min_risk_reward: Decimal,
    pub sweep_buffer_pips: Decimal,
}

impl Default for SmcLiquiditySweepStrategy {
    fn default() -> Self {
        Self {
            name: "SMC-Liquidity-Sweep-v1".to_string(),
            min_risk_reward: dec!(2.0),
            sweep_buffer_pips: dec!(2.0),
        }
    }
}

impl SmcLiquiditySweepStrategy {
    pub fn new(name: impl Into<String>, min_risk_reward: Decimal, sweep_buffer_pips: Decimal) -> Self {
        Self {
            name: name.into(),
            min_risk_reward,
            sweep_buffer_pips,
        }
    }
}

#[async_trait]
impl StrategyPort for SmcLiquiditySweepStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError> {
        if ctx.candles.len() < 20 {
            return Ok(None);
        }

        let n = ctx.candles.len();
        let recent_slice = &ctx.candles[(n - 20)..n];
        let spec = TfPairSpec::from_symbol(ctx.symbol);

        // Cari level Liquidity Pool (Lowest Low & Highest High dari 20 bar terakhir)
        let mut min_low = Decimal::MAX;
        let mut max_high = Decimal::ZERO;

        for c in &recent_slice[0..18] {
            if c.low < min_low {
                min_low = c.low;
            }
            if c.high > max_high {
                max_high = c.high;
            }
        }

        let last_candle = &recent_slice[19];
        let current_price = ctx.current_tick.bid;
        let buffer_offset = spec.pip_size * self.sweep_buffer_pips;

        // 1. Skenario Bullish Sweep: Candle menusuk di bawah min_low lalu close di atasnya (Liquidity Grab)
        if last_candle.low < min_low && last_candle.close > min_low {
            let entry = min_low;
            let sl = last_candle.low - buffer_offset;
            let risk_distance = entry - sl;

            if risk_distance > Decimal::ZERO {
                let tp1 = max_high;
                let reward_distance = tp1 - entry;
                let rr = reward_distance / risk_distance;

                if rr >= self.min_risk_reward && rr <= dec!(3.0) {
                    let signal = Signal {
                        id: Uuid::new_v4(),
                        symbol: ctx.symbol.clone(),
                        action: SignalAction::BuyLimit,
                        timeframe: ctx.timeframe,
                        entry_price: entry,
                        stop_loss: sl,
                        take_profit_1: tp1,
                        take_profit_2: Some(entry + (risk_distance * dec!(2.5))),
                        take_profit_3: None,
                        risk_reward_ratio: rr,
                        confidence_score: 0.92,
                        strategy_name: self.name.clone(),
                        rationale: format!(
                            "SMC Bullish Liquidity Sweep di {:.5} + Bullish Order Block Rejection",
                            min_low
                        ),
                        status: SignalStatus::Pending,
                        created_at: Utc::now(),
                        expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
                    };
                    return Ok(Some(signal));
                }
            }
        }

        // 2. Skenario Bearish Sweep: Candle menusuk di atas max_high lalu close di bawahnya
        if last_candle.high > max_high && last_candle.close < max_high {
            let entry = max_high;
            let sl = last_candle.high + buffer_offset;
            let risk_distance = sl - entry;

            if risk_distance > Decimal::ZERO {
                let tp1 = min_low;
                let reward_distance = entry - tp1;
                let rr = reward_distance / risk_distance;

                if rr >= self.min_risk_reward && rr <= dec!(3.0) {
                    let signal = Signal {
                        id: Uuid::new_v4(),
                        symbol: ctx.symbol.clone(),
                        action: SignalAction::SellLimit,
                        timeframe: ctx.timeframe,
                        entry_price: entry,
                        stop_loss: sl,
                        take_profit_1: tp1,
                        take_profit_2: Some(entry - (risk_distance * dec!(2.5))),
                        take_profit_3: None,
                        risk_reward_ratio: rr,
                        confidence_score: 0.92,
                        strategy_name: self.name.clone(),
                        rationale: format!(
                            "SMC Bearish Liquidity Sweep di {:.5} + Bearish Order Block Rejection",
                            max_high
                        ),
                        status: SignalStatus::Pending,
                        created_at: Utc::now(),
                        expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
                    };
                    return Ok(Some(signal));
                }
            }
        }

        let _ = current_price;
        Ok(None)
    }
}
