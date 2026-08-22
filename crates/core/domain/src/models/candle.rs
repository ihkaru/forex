use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MarketDataSource {
    #[default]
    DukascopyEcn,
    Mt5BrokerLive,
    CtraderOpenApi,
    SyntheticTest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub source: MarketDataSource,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: Symbol,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub source: MarketDataSource,
    pub bid: Decimal,
    pub ask: Decimal,
}

impl Tick {
    pub fn spread(&self) -> Decimal {
        self.ask - self.bid
    }

    pub fn mid_price(&self) -> Decimal {
        (self.bid + self.ask) / rust_decimal_macros::dec!(2)
    }
}
