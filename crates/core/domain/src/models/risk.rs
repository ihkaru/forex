use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskProfile {
    pub max_risk_per_trade_percent: Decimal,
    pub min_risk_reward_ratio: Decimal,
    pub max_open_drawdown_percent: Decimal,
    pub max_spread_pips: Decimal,
}

impl Default for RiskProfile {
    fn default() -> Self {
        Self {
            max_risk_per_trade_percent: rust_decimal_macros::dec!(1.0), // 1% per trade
            min_risk_reward_ratio: rust_decimal_macros::dec!(2.0),      // 1:2 Minimum
            max_open_drawdown_percent: rust_decimal_macros::dec!(5.0),  // 5% max daily drawdown
            max_spread_pips: rust_decimal_macros::dec!(2.5),
        }
    }
}

impl RiskProfile {
    pub fn from_config(config: &crate::models::config::RiskConfig) -> Self {
        Self {
            max_risk_per_trade_percent: config.max_risk_per_trade_percent,
            min_risk_reward_ratio: config.min_risk_reward_ratio,
            max_open_drawdown_percent: config.max_open_drawdown_percent,
            max_spread_pips: config.max_spread_pips,
        }
    }

    pub fn calculate_risk_reward(
        &self,
        entry: Decimal,
        stop_loss: Decimal,
        take_profit: Decimal,
    ) -> Option<Decimal> {
        let risk = (entry - stop_loss).abs();
        let reward = (take_profit - entry).abs();

        if risk.is_zero() {
            return None;
        }

        Some(reward / risk)
    }

    pub fn is_risk_reward_acceptable(
        &self,
        entry: Decimal,
        stop_loss: Decimal,
        take_profit: Decimal,
    ) -> bool {
        if let Some(rr) = self.calculate_risk_reward(entry, stop_loss, take_profit) {
            rr >= self.min_risk_reward_ratio
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rstest::rstest;
    use rust_decimal_macros::dec;

    #[rstest]
    #[case(dec!(1.0850), dec!(1.0830), dec!(1.0890), true)] // R:R 1:2 (Acceptable)
    #[case(dec!(1.0850), dec!(1.0830), dec!(1.0910), true)] // R:R 1:3 (Acceptable)
    #[case(dec!(1.0850), dec!(1.0845), dec!(1.0855), false)] // R:R 1:1 (Rejected < 1:2)
    #[case(dec!(1.0850), dec!(1.0850), dec!(1.0890), false)] // SL == Entry (Division by zero / Rejected)
    fn test_risk_reward_matrix(
        #[case] entry: Decimal,
        #[case] sl: Decimal,
        #[case] tp: Decimal,
        #[case] expected: bool,
    ) {
        let profile = RiskProfile::default();
        let result = profile.is_risk_reward_acceptable(entry, sl, tp);
        assert_eq!(result, expected);
    }

    proptest! {
        #[test]
        fn prop_test_non_zero_risk_never_panics(
            entry_raw in 10000i64..20000i64,
            sl_diff in 10i64..500i64,
            tp_diff in 10i64..1000i64,
        ) {
            let entry = Decimal::new(entry_raw, 4);
            let sl = Decimal::new(entry_raw - sl_diff, 4);
            let tp = Decimal::new(entry_raw + tp_diff, 4);

            let profile = RiskProfile::default();
            let rr = profile.calculate_risk_reward(entry, sl, tp);
            prop_assert!(rr.is_some());
            prop_assert!(rr.unwrap() > Decimal::ZERO);
        }
    }
}
