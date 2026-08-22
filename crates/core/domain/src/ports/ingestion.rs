use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::models::{Symbol, Timeframe};

/// Permintaan ingesti data pasar historis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionRequest {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub from_year: i32,
    pub to_year: i32,
}

/// Hasil laporan ingesti data pasar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestionResult {
    pub status: String,
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub total_candles: usize,
    pub new_candles_added: usize,
    pub first_timestamp: Option<DateTime<Utc>>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub tier: usize,
    pub multiplier: Decimal,
    pub min_sl_tp_pips: Decimal,
}

/// Status ketersediaan data untuk satu simbol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolStatusDto {
    pub symbol: Symbol,
    pub tier: usize,
    pub multiplier: Decimal,
    pub pip_size: Decimal,
    pub min_sl_tp_pips: Decimal,
    pub max_sl_tp_pips: Decimal,
    pub candle_count: usize,
    pub first_timestamp: Option<DateTime<Utc>>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub is_available: bool,
}

/// Port Ingesti Data Pasar (Interface-First Pattern)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MarketIngestionPort: Send + Sync {
    /// Menjalankan pipeline ingesti data historis secara idempoten
    async fn ingest_symbol(&self, req: IngestionRequest) -> Result<IngestionResult, DomainError>;

    /// Mendapatkan status seluruh simbol yang terdaftar beserta statistik datanya
    async fn list_available_symbols(&self) -> Result<Vec<SymbolStatusDto>, DomainError>;
}
