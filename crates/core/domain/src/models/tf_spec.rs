use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::symbol::Symbol;

pub use super::tf_compliance::TfComplianceGuard;
pub use super::tf_qualification::{MonthlyQualificationGuard, MonthlyQualificationStatus};

#[cfg(test)]
mod tf_spec_test;

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
