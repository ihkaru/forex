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

/// Model Kuantitatif Kelly Criterion (Discrete & Continuous Merton Model)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KellyCriterion {
    /// Fraksi alokasi (misal 0.25 untuk Quarter Kelly, 0.5 untuk Half Kelly, 1.0 untuk Full Kelly)
    pub kelly_fraction: Decimal,
}

impl Default for KellyCriterion {
    fn default() -> Self {
        Self {
            // Standar institusional 2026: Quarter Kelly (0.25) untuk meminimalkan drawdown
            kelly_fraction: rust_decimal_macros::dec!(0.25),
        }
    }
}

impl KellyCriterion {
    pub fn new(kelly_fraction: Decimal) -> Self {
        Self {
            kelly_fraction: kelly_fraction.clamp(
                rust_decimal_macros::dec!(0.01),
                rust_decimal_macros::dec!(1.0),
            ),
        }
    }

    /// Discrete Kelly Criterion:
    /// f* = (p * b - (1 - p)) / b
    /// p: Win Rate (0.0 s.d. 1.0)
    /// b: Payoff Ratio (Take Profit / Stop Loss)
    pub fn calculate_discrete_kelly(
        &self,
        win_rate: Decimal,
        payoff_ratio: Decimal,
    ) -> Option<Decimal> {
        if payoff_ratio <= Decimal::ZERO || win_rate < Decimal::ZERO || win_rate > Decimal::ONE {
            return None;
        }

        let loss_rate = Decimal::ONE - win_rate;
        let expected_edge = (win_rate * payoff_ratio) - loss_rate;

        if expected_edge <= Decimal::ZERO {
            return Some(Decimal::ZERO); // Edge negatif -> Jangan ambil risiko
        }

        let full_kelly = expected_edge / payoff_ratio;
        let fractional_kelly = full_kelly * self.kelly_fraction;

        Some(fractional_kelly.clamp(Decimal::ZERO, Decimal::ONE))
    }

    /// Continuous Kelly Criterion (Merton Portfolio Problem):
    /// f* = (mu - r) / sigma^2
    /// mu: Expected instantaneous return / drift rate
    /// r: Risk-free rate
    /// variance_sigma_sq: Variance of returns (sigma^2)
    pub fn calculate_continuous_kelly(
        &self,
        drift_mu: Decimal,
        risk_free_r: Decimal,
        variance_sigma_sq: Decimal,
    ) -> Option<Decimal> {
        if variance_sigma_sq <= Decimal::ZERO {
            return None;
        }

        let excess_return = drift_mu - risk_free_r;
        if excess_return <= Decimal::ZERO {
            return Some(Decimal::ZERO);
        }

        let full_continuous_kelly = excess_return / variance_sigma_sq;
        let scaled_kelly = full_continuous_kelly * self.kelly_fraction;

        Some(scaled_kelly.max(Decimal::ZERO))
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

    #[test]
    fn test_discrete_kelly_criterion() {
        let kelly = KellyCriterion::new(dec!(0.25)); // Quarter Kelly

        // Kasus 1: Win Rate 50%, Payoff 1.49 (R:R 1:1.49)
        // Expected edge = (0.5 * 1.49) - 0.5 = 0.745 - 0.5 = 0.245
        // Full Kelly = 0.245 / 1.49 ≈ 0.1644295
        // Quarter Kelly = 0.1644295 * 0.25 ≈ 0.041107 (4.11%)
        let f_star = kelly
            .calculate_discrete_kelly(dec!(0.50), dec!(1.49))
            .unwrap();
        assert!(f_star > dec!(0.040) && f_star < dec!(0.042));

        // Kasus 2: Win Rate 30%, Payoff 1.0 -> Negative Edge -> Return 0
        let f_neg = kelly
            .calculate_discrete_kelly(dec!(0.30), dec!(1.0))
            .unwrap();
        assert_eq!(f_neg, Decimal::ZERO);
    }

    #[test]
    fn test_continuous_kelly_criterion() {
        let kelly = KellyCriterion::new(dec!(0.50)); // Half Kelly

        // Drift mu = 0.12 (12%), r = 0.02 (2%), Variance sigma^2 = 0.04 (volatility 20%)
        // Full Kelly = (0.12 - 0.02) / 0.04 = 0.10 / 0.04 = 2.5
        // Half Kelly = 2.5 * 0.5 = 1.25
        let f_cont = kelly
            .calculate_continuous_kelly(dec!(0.12), dec!(0.02), dec!(0.04))
            .unwrap();
        assert_eq!(f_cont, dec!(1.25));

        // Excess return <= 0 -> Return 0
        let f_zero = kelly
            .calculate_continuous_kelly(dec!(0.01), dec!(0.02), dec!(0.04))
            .unwrap();
        assert_eq!(f_zero, Decimal::ZERO);
    }
}
