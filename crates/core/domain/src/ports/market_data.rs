use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::errors::DomainError;
use crate::models::{Candle, Symbol, Tick, Timeframe};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MarketDataPort: Send + Sync {
    /// Ambil snapshot tick harga terkini
    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError>;

    /// Ambil n bar candle terakhir untuk timeframe tertentu
    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError>;

    /// Ambil candle historis dalam rentang waktu tertentu untuk backtesting
    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError>;
}
