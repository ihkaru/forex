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
            swap_long: dec!(-7.0),  // Estimasi konservatif broker retail: -0.7 pips/malam
            swap_short: dec!(-7.0), // Estimasi konservatif broker retail: -0.7 pips/malam
            swap_rollover_3days: 3,
            stop_level_pts: 25,     // Broker retail stop level: 2.5 pips
            freeze_level_pts: 10,   // Broker retail freeze level: 1.0 pip
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
                swap_long: dec!(-7.0),
                swap_short: dec!(-7.0),
                swap_rollover_3days: 3,
                stop_level_pts: 25,
                freeze_level_pts: 10,
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

    /// Menghitung akumulasi beban biaya swap menginap (dalam satuan pips).
    /// Memperhitungkan posisi Buy (swap_long) vs Sell (swap_short),
    /// serta rollover 3x lipat pada hari Rabu (Triple Swap Wednesday).
    pub fn calculate_swap_pips(
        &self,
        is_buy: bool,
        open_time: chrono::DateTime<chrono::Utc>,
        close_time: chrono::DateTime<chrono::Utc>,
    ) -> Decimal {
        use chrono::Datelike;
        if open_time >= close_time {
            return Decimal::ZERO;
        }

        let swap_rate = if is_buy { self.swap_long } else { self.swap_short };
        if swap_rate == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let mut rollovers = 0;
        let mut cur = open_time;
        while cur < close_time {
            cur = cur + chrono::Duration::days(1);
            if cur <= close_time {
                // Hari Rabu malam (Kamis 00:00 server time) dikenakan 3x swap standar broker forex
                if cur.weekday() == chrono::Weekday::Thu {
                    rollovers += 3;
                } else if cur.weekday() != chrono::Weekday::Sat && cur.weekday() != chrono::Weekday::Sun {
                    rollovers += 1;
                }
            }
        }

        if rollovers == 0 {
            return Decimal::ZERO;
        }

        // Swap rate dari broker biasanya bernilai negatif jika biaya (-15.0 pts)
        // Kita kembalikan nilai biaya positif (cost in pips) yang memotong PnL
        let pip_mult = self.points_to_pips(1);
        -(swap_rate * Decimal::from(rollovers) * pip_mult)
    }
}
