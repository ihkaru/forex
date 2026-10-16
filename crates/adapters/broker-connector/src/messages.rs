use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Kapasitas maksimal bounded circular buffer untuk lilin pasar (menjaga footprint memori konstan O(1))
pub const MAX_CANDLE_CAPACITY: usize = 500;

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
        time_gmt: i64,
    },
}
