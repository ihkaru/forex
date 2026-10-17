use crate::models::symbol::Symbol;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Spesifikasi Kontrak dan Aturan Perdagangan Broker (MT4/MT5/cTrader)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrokerSymbolSpec {
    pub symbol: Symbol,
    pub raw_ticker: String,
    pub digits: u32,
    pub point: Decimal,
    pub tick_value: Decimal,
    pub tick_size: Decimal,
    pub contract_size: Decimal,
    pub swap_long: Decimal,
    pub swap_short: Decimal,
    pub swap_rollover_3days: u32, // Default 3 (Rabu)
    pub stop_level_pts: u32,
    pub freeze_level_pts: u32,
    pub min_lot: Decimal,
    pub lot_step: Decimal,
    pub max_lot: Decimal,
    pub margin_required: Decimal,
}

impl Default for BrokerSymbolSpec {
    fn default() -> Self {
        Self {
            symbol: Symbol::new("EUR", "USD"),
            raw_ticker: "EURUSD".to_string(),
            digits: 5,
            point: dec!(0.00001),
            tick_value: dec!(1.0),
            tick_size: dec!(0.00001),
            contract_size: dec!(100000.0),
            swap_long: Decimal::ZERO,
            swap_short: Decimal::ZERO,
            swap_rollover_3days: 3,
            stop_level_pts: 10,
            freeze_level_pts: 0,
            min_lot: dec!(0.01),
            lot_step: dec!(0.01),
            max_lot: dec!(100.0),
            margin_required: dec!(1000.0),
        }
    }
}

impl BrokerSymbolSpec {
    pub fn new(symbol: Symbol, raw_ticker: impl Into<String>) -> Self {
        let raw = raw_ticker.into();
        let is_gold = raw.to_uppercase().contains("XAU") || symbol.base == "XAU";
        let is_jpy = symbol.quote == "JPY";

        if is_gold {
            Self {
                symbol,
                raw_ticker: raw,
                digits: 2,
                point: dec!(0.01),
                tick_value: dec!(1.0),
                tick_size: dec!(0.01),
                contract_size: dec!(100.0), // 100 troy oz
                swap_long: dec!(-15.0),
                swap_short: dec!(5.0),
                swap_rollover_3days: 3,
                stop_level_pts: 20,
                freeze_level_pts: 0,
                min_lot: dec!(0.01),
                lot_step: dec!(0.01),
                max_lot: dec!(50.0),
                margin_required: dec!(2000.0),
            }
        } else if is_jpy {
            Self {
                symbol,
                raw_ticker: raw,
                digits: 3,
                point: dec!(0.001),
                tick_value: dec!(0.68),
                tick_size: dec!(0.001),
                contract_size: dec!(100000.0),
                swap_long: Decimal::ZERO,
                swap_short: Decimal::ZERO,
                swap_rollover_3days: 3,
                stop_level_pts: 15,
                freeze_level_pts: 0,
                min_lot: dec!(0.01),
                lot_step: dec!(0.01),
                max_lot: dec!(100.0),
                margin_required: dec!(1000.0),
            }
        } else {
            Self {
                symbol,
                raw_ticker: raw,
                ..Default::default()
            }
        }
    }

    /// Konversi poin broker ke pips standar
    #[inline]
    pub fn points_to_pips(&self, points: u32) -> Decimal {
        if self.digits == 3 || self.digits == 5 {
            Decimal::from(points) / dec!(10.0)
        } else {
            Decimal::from(points)
        }
    }
}
