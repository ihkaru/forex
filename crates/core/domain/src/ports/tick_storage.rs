use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::errors::DomainError;
use crate::models::{MarketDataSource, Symbol, Tick};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TickStoragePort: Send + Sync {
    /// Simpan batch tick ke persistent storage (TimescaleDB compressed hypertable / Parquet columnar)
    async fn save_ticks(&self, ticks: &[Tick]) -> Result<(), DomainError>;

    /// Ambil ticks historis untuk rentang waktu tertentu
    async fn get_ticks(
        &self,
        symbol: &Symbol,
        source: MarketDataSource,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DomainError>;

    /// Ambil tick watermark tertinggi
    async fn get_tick_high_watermark(
        &self,
        symbol: &Symbol,
        source: MarketDataSource,
    ) -> Result<Option<DateTime<Utc>>, DomainError>;
}
