use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::{Candle, MarketDataSource, Order, Signal, Symbol, Timeframe};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StoragePort: Send + Sync {
    // Sinyal
    async fn save_signal(&self, signal: &Signal) -> Result<(), DomainError>;
    async fn get_signal(&self, id: Uuid) -> Result<Option<Signal>, DomainError>;
    async fn get_active_signals(&self) -> Result<Vec<Signal>, DomainError>;

    // Candle & Order
    async fn save_candles(&self, candles: &[Candle]) -> Result<(), DomainError>;
    async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError>;
    async fn get_high_watermark(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<Option<DateTime<Utc>>, DomainError>;
    async fn save_order(&self, order: &Order) -> Result<(), DomainError>;
}
