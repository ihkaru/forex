use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Kapasitas maksimal bounded circular buffer untuk lilin pasar (menjaga footprint memori konstan O(1))
pub const MAX_CANDLE_CAPACITY: usize = 500;

fn default_rollover() -> u32 {
    3
}

/// Pesan mentah yang dikirimkan oleh MQL4/MQL5 EA Socket Bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Mt5SocketMessage {
    #[serde(rename = "TICK")]
    Tick {
        symbol: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        server: Option<String>,
        bid: Decimal,
        ask: Decimal,
        spread_pts: u32,
        #[serde(default)]
        volume: Option<Decimal>,
        time_gmt: i64,
    },
    #[serde(rename = "BAR")]
    Bar {
        symbol: String,
        #[serde(default)]
        source: Option<String>,
        timeframe: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        #[serde(default)]
        spread_pts: Option<u32>,
        #[serde(default)]
        real_volume: Option<Decimal>,
        time_gmt: i64,
    },
    #[serde(rename = "SPEC")]
    Spec {
        symbol: String,
        #[serde(default)]
        source: Option<String>,
        digits: u32,
        point: Decimal,
        tick_value: Decimal,
        tick_size: Decimal,
        contract_size: Decimal,
        swap_long: Decimal,
        swap_short: Decimal,
        #[serde(default = "default_rollover")]
        swap_rollover_3days: u32,
        stop_level_pts: u32,
        freeze_level_pts: u32,
        min_lot: Decimal,
        lot_step: Decimal,
        max_lot: Decimal,
        margin_required: Decimal,
    },
}
