use chrono::{DateTime, Datelike, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::models::signal::{Signal, SignalAction};
use crate::models::symbol::Symbol;

/// Klasifikasi Tier Pair Resmi Traders Family (Update 2025/2026)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairTier {
    /// Tier 1: NZDUSD, AUDUSD, EURGBP, USDCHF (Value Multiplier 2.0x)
    Tier1,
    /// Tier 2: USDCAD, EURUSD, GBPUSD, NZDJPY, CADJPY, AUDJPY (Value Multiplier 1.5x)
    Tier2,
    /// Tier 3: CHFJPY, USDJPY, EURJPY, GBPJPY, EURNZD (Value Multiplier 1.0x)
    Tier3,
    /// Tier 4: XAUUSD / Gold (Value Multiplier 0.5x)
    Tier4,
}

/// Spesifikasi Regulasi Pembuatan Sinyal per Simbol di Aplikasi Traders Family
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TfPairSpec {
    pub symbol: Symbol,
    pub tier: PairTier,
    pub value_multiplier: Decimal,
    pub min_pending_distance_pips: Decimal,
    pub min_sl_tp_pips: Decimal,
    pub max_sl_tp_pips: Decimal,
    pub min_same_direction_gap_pips: Decimal,
    pub pip_size: Decimal, // 0.00010 for 5-digit forex, 0.010 for JPY & Gold
}

impl TfPairSpec {
    /// Mendapatkan spesifikasi resmi berdasarkan pasangan mata uang
    pub fn from_symbol(symbol: &Symbol) -> Self {
        let compact = symbol.to_compact_string();
        match compact.as_str() {
            // TIER 1: Value 2.0x, Min Pending/SL/TP 10.0, Max SL/TP 200.0, Gap 50.0
            "NZDUSD" | "AUDUSD" | "EURGBP" | "USDCHF" => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier1,
                value_multiplier: dec!(2.0),
                min_pending_distance_pips: dec!(10.0),
                min_sl_tp_pips: dec!(10.0),
                max_sl_tp_pips: dec!(200.0),
                min_same_direction_gap_pips: dec!(50.0),
                pip_size: dec!(0.00010),
            },

            // TIER 2: Value 1.5x, Min Pending/SL/TP 15.0, Max SL/TP 300.0, Gap 75.0
            "USDCAD" | "EURUSD" | "GBPUSD" => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier2,
                value_multiplier: dec!(1.5),
                min_pending_distance_pips: dec!(15.0),
                min_sl_tp_pips: dec!(15.0),
                max_sl_tp_pips: dec!(300.0),
                min_same_direction_gap_pips: dec!(75.0),
                pip_size: dec!(0.00010),
            },
            "NZDJPY" | "CADJPY" | "AUDJPY" => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier2,
                value_multiplier: dec!(1.5),
                min_pending_distance_pips: dec!(15.0),
                min_sl_tp_pips: dec!(15.0),
                max_sl_tp_pips: dec!(300.0),
                min_same_direction_gap_pips: dec!(75.0),
                pip_size: dec!(0.010),
            },

            // TIER 3: Value 1.0x, Min Pending/SL/TP 20.0, Max SL/TP 400.0, Gap 100.0
            "USDJPY" | "EURJPY" | "GBPJPY" | "CHFJPY" => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier3,
                value_multiplier: dec!(1.0),
                min_pending_distance_pips: dec!(20.0),
                min_sl_tp_pips: dec!(20.0),
                max_sl_tp_pips: dec!(400.0),
                min_same_direction_gap_pips: dec!(100.0),
                pip_size: dec!(0.010),
            },
            "EURNZD" => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier3,
                value_multiplier: dec!(1.0),
                min_pending_distance_pips: dec!(20.0),
                min_sl_tp_pips: dec!(20.0),
                max_sl_tp_pips: dec!(400.0),
                min_same_direction_gap_pips: dec!(100.0),
                pip_size: dec!(0.00010),
            },

            // TIER 4: XAUUSD (Gold): Value 0.5x, Min Pending/SL/TP 30.0, Max SL/TP 500.0, Gap 100.0
            "XAUUSD" => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier4,
                value_multiplier: dec!(0.5),
                min_pending_distance_pips: dec!(30.0),
                min_sl_tp_pips: dec!(30.0),
                max_sl_tp_pips: dec!(500.0),
                min_same_direction_gap_pips: dec!(100.0),
                pip_size: dec!(0.10),
            },

            // Default fallback
            _ => Self {
                symbol: symbol.clone(),
                tier: PairTier::Tier2,
                value_multiplier: dec!(1.0),
                min_pending_distance_pips: dec!(15.0),
                min_sl_tp_pips: dec!(15.0),
                max_sl_tp_pips: dec!(300.0),
                min_same_direction_gap_pips: dec!(75.0),
                pip_size: dec!(0.00010),
            },
        }
    }

    /// Konversi selisih harga menjadi pips
    pub fn price_diff_to_pips(&self, price_diff: Decimal) -> Decimal {
        if self.pip_size.is_zero() {
            Decimal::ZERO
        } else {
            price_diff.abs() / self.pip_size
        }
    }

    /// Konversi pips menjadi Valued Pips (VP)
    pub fn pips_to_valued_pips(&self, pips: Decimal) -> Decimal {
        pips * self.value_multiplier
    }
}

/// Guard Validator Kepatuhan Sinyal terhadap Kebijakan Traders Family (100% Zero-Banned Guarantee)
pub struct TfComplianceGuard;

impl TfComplianceGuard {
    /// Validasi menyeluruh terhadap sinyal sebelum dipublikasikan ke channel Traders Family
    pub fn validate_signal(signal: &Signal) -> Result<(), DomainError> {
        // 1. Wajib Pending Order (Instant execution dilarang di TF App)
        match signal.action {
            SignalAction::BuyLimit
            | SignalAction::SellLimit
            | SignalAction::BuyStop
            | SignalAction::SellStop => {}
            SignalAction::Buy | SignalAction::Sell | SignalAction::ClosePosition => {
                return Err(DomainError::ValidationError(
                    "Sinyal TF wajib bertipe Pending Order (BuyLimit, SellLimit, BuyStop, SellStop)".to_string(),
                ));
            }
        }

        let spec = TfPairSpec::from_symbol(&signal.symbol);

        // 2. Hitung jarak SL dan TP dalam pips
        let sl_distance = (signal.entry_price - signal.stop_loss).abs();
        let tp_distance = (signal.take_profit_1 - signal.entry_price).abs();

        let sl_pips = spec.price_diff_to_pips(sl_distance);
        let tp_pips = spec.price_diff_to_pips(tp_distance);

        // 3. Validasi Batas Minimal & Maksimal Stop Loss
        if sl_pips < spec.min_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Stop Loss ({:.1} pips) di bawah batas minimal tier {} ({:.1} pips)",
                sl_pips, spec.symbol, spec.min_sl_tp_pips
            )));
        }
        if sl_pips > spec.max_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Stop Loss ({:.1} pips) melebihi batas maksimal tier {} ({:.1} pips)",
                sl_pips, spec.symbol, spec.max_sl_tp_pips
            )));
        }

        // 4. Validasi Batas Minimal & Maksimal Take Profit
        if tp_pips < spec.min_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Take Profit ({:.1} pips) di bawah batas minimal tier {} ({:.1} pips)",
                tp_pips, spec.symbol, spec.min_sl_tp_pips
            )));
        }
        if tp_pips > spec.max_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Take Profit ({:.1} pips) melebihi batas maksimal tier {} ({:.1} pips)",
                tp_pips, spec.symbol, spec.max_sl_tp_pips
            )));
        }

        // 5. Validasi Maksimal Rasio Risk:Reward (Maksimal 1:3.0)
        let rr = tp_distance / sl_distance;
        if rr > dec!(3.00) {
            return Err(DomainError::ValidationError(format!(
                "Rasio Risk:Reward (1:{:.2}) melebihi batas maksimal TF (1:3.00)",
                rr
            )));
        }
        if rr < dec!(1.00) {
            return Err(DomainError::ValidationError(format!(
                "Rasio Risk:Reward (1:{:.2}) tidak sehat (wajib minimal 1:1.00)",
                rr
            )));
        }

        // 6. Validasi Stop Loss maksimal 1.5x dari Take Profit
        if sl_pips > (tp_pips * dec!(1.5)) {
            return Err(DomainError::ValidationError(format!(
                "Stop Loss ({:.1} pips) melebihi 1.5x Take Profit ({:.1} pips)",
                sl_pips, tp_pips
            )));
        }

        // 7. Validasi Masa Kadaluwarsa (Expired Time: 1 jam s.d. 48 jam / 96 jam Jumat)
        if let Some(expires_at) = signal.expires_at {
            Self::validate_expiration(signal.created_at, expires_at)?;
        }

        Ok(())
    }

    /// Validasi durasi kedaluwarsa sesuai hari pembuatan
    pub fn validate_expiration(
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let duration_hours = (expires_at - created_at).num_hours();
        let is_friday = created_at.weekday() == chrono::Weekday::Fri;
        let max_hours = if is_friday { 96 } else { 48 };

        if duration_hours < 1 {
            return Err(DomainError::ValidationError(
                "Durasi expired sinyal minimal adalah 1 jam".to_string(),
            ));
        }
        if duration_hours > max_hours {
            return Err(DomainError::ValidationError(format!(
                "Durasi expired sinyal ({} jam) melebihi batas maksimal ({} jam)",
                duration_hours, max_hours
            )));
        }

        Ok(())
    }
}

/// Status Evaluasi Kualifikasi Bulanan Analis Traders Family
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyQualificationStatus {
    pub settled_signals_count: usize,
    pub min_settled_signals_required: usize,
    pub total_valued_pips: Decimal,
    pub min_valued_pips_required: Decimal,
    pub is_settled_volume_passed: bool,
    pub is_valued_pips_passed: bool,
    pub is_fully_qualified: bool,
    pub rejection_reasons: Vec<String>,
}

/// Guard Domain Invariant untuk mengevaluasi & menjamin kualifikasi bulanan analis TF
pub struct MonthlyQualificationGuard;

impl MonthlyQualificationGuard {
    /// Batas mutlak minimal sinyal settled per bulan di platform Traders Family (Wajib > 20 sinyal/bulan)
    pub const MIN_SETTLED_SIGNALS: usize = 20;
    /// Batas mutlak target Valued Pips per bulan di platform Traders Family
    pub const MIN_VALUED_PIPS: Decimal = dec!(300.0);

    /// Evaluasi apakah performa bulan berjalan memenuhi syarat kualifikasi TF
    pub fn evaluate_month(
        settled_signals: usize,
        total_valued_pips: Decimal,
    ) -> MonthlyQualificationStatus {
        let is_settled_volume_passed = settled_signals >= Self::MIN_SETTLED_SIGNALS;
        let is_valued_pips_passed = total_valued_pips >= Self::MIN_VALUED_PIPS;
        let is_fully_qualified = is_settled_volume_passed && is_valued_pips_passed;

        let mut rejection_reasons = Vec::new();
        if !is_settled_volume_passed {
            rejection_reasons.push(format!(
                "Volume sinyal settled ({} sinyal) kurang dari batas minimal TF ({} sinyal)",
                settled_signals,
                Self::MIN_SETTLED_SIGNALS
            ));
        }
        if !is_valued_pips_passed {
            rejection_reasons.push(format!(
                "Akumulasi Valued Pips ({:.1} VP) kurang dari target kelulusan TF ({:.1} VP)",
                total_valued_pips,
                Self::MIN_VALUED_PIPS
            ));
        }

        MonthlyQualificationStatus {
            settled_signals_count: settled_signals,
            min_settled_signals_required: Self::MIN_SETTLED_SIGNALS,
            total_valued_pips,
            min_valued_pips_required: Self::MIN_VALUED_PIPS,
            is_settled_volume_passed,
            is_valued_pips_passed,
            is_fully_qualified,
            rejection_reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::candle::Timeframe;
    use crate::models::signal::SignalStatus;
    use uuid::Uuid;

    #[test]
    fn test_tier1_spec_valued_pips_multiplication() {
        let spec = TfPairSpec::from_symbol(&Symbol::new("NZD", "USD"));
        assert_eq!(spec.tier, PairTier::Tier1);
        assert_eq!(spec.value_multiplier, dec!(2.0));

        let vp = spec.pips_to_valued_pips(dec!(50.0));
        assert_eq!(vp, dec!(100.0)); // 50 pips * 2.0 = 100 VP
    }

    #[test]
    fn test_tier4_gold_spec() {
        let spec = TfPairSpec::from_symbol(&Symbol::new("XAU", "USD"));
        assert_eq!(spec.tier, PairTier::Tier4);
        assert_eq!(spec.value_multiplier, dec!(0.5));
        assert_eq!(spec.max_sl_tp_pips, dec!(500.0));

        let vp = spec.pips_to_valued_pips(dec!(100.0));
        assert_eq!(vp, dec!(50.0)); // 100 pips * 0.5 = 50 VP
    }

    #[test]
    fn test_compliance_guard_accepts_valid_pending_order() {
        let signal = Signal {
            id: Uuid::new_v4(),
            symbol: Symbol::new("EUR", "USD"),
            action: SignalAction::BuyLimit,
            timeframe: Timeframe::H1,
            entry_price: dec!(1.08500),
            stop_loss: dec!(1.08200),     // 30 pips SL
            take_profit_1: dec!(1.09100), // 60 pips TP (R:R 1:2.0)
            take_profit_2: None,
            take_profit_3: None,
            risk_reward_ratio: dec!(2.0),
            confidence_score: 0.9,
            strategy_name: "Pola-N".to_string(),
            rationale: "Retest Higher Low".to_string(),
            status: SignalStatus::Pending,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
        };

        let result = TfComplianceGuard::validate_signal(&signal);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compliance_guard_rejects_instant_execution() {
        let signal = Signal {
            id: Uuid::new_v4(),
            symbol: Symbol::new("EUR", "USD"),
            action: SignalAction::Buy, // Rejected! Wajib pending order
            timeframe: Timeframe::H1,
            entry_price: dec!(1.08500),
            stop_loss: dec!(1.08200),
            take_profit_1: dec!(1.09100),
            take_profit_2: None,
            take_profit_3: None,
            risk_reward_ratio: dec!(2.0),
            confidence_score: 0.9,
            strategy_name: "Pola-N".to_string(),
            rationale: "Instant entry".to_string(),
            status: SignalStatus::Active,
            created_at: Utc::now(),
            expires_at: None,
        };

        let result = TfComplianceGuard::validate_signal(&signal);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Pending Order"));
    }

    #[test]
    fn test_compliance_guard_rejects_rr_exceeding_1_to_3() {
        let signal = Signal {
            id: Uuid::new_v4(),
            symbol: Symbol::new("EUR", "USD"),
            action: SignalAction::BuyLimit,
            timeframe: Timeframe::H1,
            entry_price: dec!(1.08500),
            stop_loss: dec!(1.08300),     // 20 pips SL
            take_profit_1: dec!(1.09300), // 80 pips TP (R:R 1:4.0 -> Rejected > 1:3.0)
            take_profit_2: None,
            take_profit_3: None,
            risk_reward_ratio: dec!(4.0),
            confidence_score: 0.9,
            strategy_name: "Pola-N".to_string(),
            rationale: "Greedy TP".to_string(),
            status: SignalStatus::Pending,
            created_at: Utc::now(),
            expires_at: None,
        };

        let result = TfComplianceGuard::validate_signal(&signal);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1:3.00"));
    }

    #[test]
    fn test_monthly_qualification_guard_success_and_failure_cases() {
        // Kasus 1: Lulus Sempurna (>= 20 settled signals & >= 300 VP)
        let status_pass = MonthlyQualificationGuard::evaluate_month(25, dec!(450.0));
        assert!(status_pass.is_fully_qualified);
        assert!(status_pass.is_settled_volume_passed);
        assert!(status_pass.is_valued_pips_passed);
        assert!(status_pass.rejection_reasons.is_empty());

        // Kasus 2: Kurang Trade Settled (< 20 signals, e.g. 12)
        let status_low_volume = MonthlyQualificationGuard::evaluate_month(12, dec!(350.0));
        assert!(!status_low_volume.is_fully_qualified);
        assert!(!status_low_volume.is_settled_volume_passed);
        assert!(status_low_volume.is_valued_pips_passed);
        assert_eq!(status_low_volume.rejection_reasons.len(), 1);
        assert!(
            status_low_volume.rejection_reasons[0].contains("Volume sinyal settled (12 sinyal)")
        );

        // Kasus 3: Kurang Valued Pips (< 300 VP, e.g. 180 VP)
        let status_low_vp = MonthlyQualificationGuard::evaluate_month(22, dec!(180.0));
        assert!(!status_low_vp.is_fully_qualified);
        assert!(status_low_vp.is_settled_volume_passed);
        assert!(!status_low_vp.is_valued_pips_passed);
        assert_eq!(status_low_vp.rejection_reasons.len(), 1);
        assert!(status_low_vp.rejection_reasons[0].contains("Akumulasi Valued Pips (180.0 VP)"));
    }
}
