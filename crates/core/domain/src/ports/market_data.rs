use crate::errors::DomainError;
use crate::models::{Candle, CandleQuery, MarketDataSource, Symbol, Tick, Timeframe};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MarketDataPort: Send + Sync {
    /// Identitas sumber data pasar yang dilayani oleh adapter ini
    fn source(&self) -> MarketDataSource;

    /// Ambil snapshot tick harga terkini
    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError>;

    /// Ambil candle secara terstruktur dengan penegakan source (Interface-First / TV UDF)
    async fn query_candles(&self, query: &CandleQuery) -> Result<Vec<Candle>, DomainError> {
        if query.source != self.source() {
            return Err(DomainError::ValidationError(format!(
                "Mismatched source: adapter '{}' tidak dapat melayani query source '{}'",
                self.source().as_str(),
                query.source.as_str()
            )));
        }

        if let (Some(from), Some(to)) = (query.from, query.to) {
            self.get_historical_candles(&query.symbol, query.timeframe, from, to)
                .await
        } else {
            let limit = query.limit.unwrap_or(300);
            self.get_recent_candles(&query.symbol, query.timeframe, limit)
                .await
        }
    }

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
