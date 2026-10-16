use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

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
