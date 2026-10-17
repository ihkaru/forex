use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{
    BrokerSymbolSpec, Candle, MarketDataSource, Symbol, Tick, Timeframe,
};
use domain::ports::{BrokerSpecPort, MarketDataPort, StoragePort, TickStoragePort};
use std::sync::Arc;

/// StorageMarketDataAdapter
/// Jembatan agnostik yang mengadaptasi StoragePort, TickStoragePort, dan BrokerSpecPort
/// menjadi implementasi lengkap MarketDataPort untuk konsumsi BacktestService, Router, dan UI.
pub struct StorageMarketDataAdapter {
    source: MarketDataSource,
    storage: Arc<dyn StoragePort>,
    tick_storage: Option<Arc<dyn TickStoragePort>>,
    broker_spec_storage: Option<Arc<dyn BrokerSpecPort>>,
}

impl StorageMarketDataAdapter {
    pub fn new(source: MarketDataSource, storage: Arc<dyn StoragePort>) -> Self {
        Self {
            source,
            storage,
            tick_storage: None,
            broker_spec_storage: None,
        }
    }

    pub fn with_tick_and_spec_storage(
        source: MarketDataSource,
        storage: Arc<dyn StoragePort>,
        tick_storage: Option<Arc<dyn TickStoragePort>>,
        broker_spec_storage: Option<Arc<dyn BrokerSpecPort>>,
    ) -> Self {
        Self {
            source,
            storage,
            tick_storage,
            broker_spec_storage,
        }
    }
}

#[async_trait]
impl MarketDataPort for StorageMarketDataAdapter {
    fn source(&self) -> MarketDataSource {
        self.source
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        if let Some(ref ts) = self.tick_storage {
            let to = Utc::now();
            let from = to - chrono::Duration::hours(24);
            if let Ok(ticks) = ts.get_ticks(symbol, self.source, from, to).await {
                if let Some(last_tick) = ticks.last() {
                    return Ok(last_tick.clone());
                }
            }
        }

        let candles = self
            .storage
            .get_candles(symbol, Timeframe::H1, 1)
            .await?;
        if let Some(last) = candles.last() {
            let spread_offset = last.spread.unwrap_or(rust_decimal_macros::dec!(0.00012));
            return Ok(Tick {
                symbol: symbol.clone(),
                source: last.source,
                bid: last.close,
                ask: last.close + spread_offset,
                timestamp: last.timestamp,
                spread_pts: None,
                volume: None,
            });
        }

        Err(DomainError::DataUnavailable(format!(
            "Data snapshot tick untuk {} tidak tersedia di storage",
            symbol
        )))
    }

    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        self.storage.get_candles(symbol, timeframe, limit).await
    }

    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        self.storage
            .get_historical_candles_range(symbol, timeframe, self.source, from, to)
            .await
    }

    async fn get_historical_ticks(
        &self,
        symbol: &Symbol,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DomainError> {
        if let Some(ref ts) = self.tick_storage {
            ts.get_ticks(symbol, self.source, from, to).await
        } else {
            Ok(vec![])
        }
    }

    async fn get_broker_spec(&self, symbol: &Symbol) -> Option<BrokerSymbolSpec> {
        if let Some(ref bss) = self.broker_spec_storage {
            bss.get_broker_spec(symbol).await.ok().flatten()
        } else {
            None
        }
    }
}
