use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::models::{MarketDataSource, Symbol, Timeframe};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSyncReport {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub source: MarketDataSource,
    pub previous_watermark: Option<DateTime<Utc>>,
    pub new_watermark: Option<DateTime<Utc>>,
    pub synced_bars_count: usize,
    pub duration_ms: u64,
    pub is_up_to_date: bool,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DeltaSyncPort: Send + Sync {
    /// Menjalankan incremental delta sync untuk 1 pair dan timeframe
    async fn sync_pair_delta(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<DeltaSyncReport, DomainError>;

    /// Menjalankan incremental delta sync untuk sekumpulan pair
    async fn sync_all_pairs(
        &self,
        symbols: &[Symbol],
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<Vec<DeltaSyncReport>, DomainError>;
}
