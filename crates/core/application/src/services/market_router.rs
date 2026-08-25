use std::collections::HashMap;
use std::sync::Arc;

use domain::errors::DomainError;
use domain::models::{Candle, CandleQuery, MarketDataSource, Symbol, Tick, Timeframe};
use domain::ports::MarketDataPort;

/// MarketDataRouterService
/// Layanan Router Data Pasar Kuantitatif (Interface-First / TV UDF Inspired)
/// Memastikan setiap query lilin di-route secara deterministik ke adapter yang tepat
/// tanpa ada risiko salah data atau silent fallback.
pub struct MarketDataRouterService {
    adapters: HashMap<MarketDataSource, Arc<dyn MarketDataPort>>,
}

impl Default for MarketDataRouterService {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketDataRouterService {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Daftarkan adapter data pasar konkret
    pub fn register(&mut self, adapter: Arc<dyn MarketDataPort>) {
        let source = adapter.source();
        self.adapters.insert(source, adapter);
    }

    /// Ambil adapter untuk sumber tertentu
    pub fn get_adapter(
        &self,
        source: MarketDataSource,
    ) -> Result<Arc<dyn MarketDataPort>, DomainError> {
        self.adapters.get(&source).cloned().ok_or_else(|| {
            DomainError::AdapterError(format!(
                "Adapter data pasar untuk sumber '{}' belum terdaftar",
                source.as_str()
            ))
        })
    }

    /// Query lilin pasar dengan penegakan source secara ketat
    pub async fn query_candles(&self, query: &CandleQuery) -> Result<Vec<Candle>, DomainError> {
        let adapter = self.get_adapter(query.source)?;
        adapter.query_candles(query).await
    }

    /// Snapshot tick terkini untuk simbol dan sumber tertentu
    pub async fn get_latest_tick(
        &self,
        symbol: &Symbol,
        source: MarketDataSource,
    ) -> Result<Tick, DomainError> {
        let adapter = self.get_adapter(source)?;
        adapter.get_latest_tick(symbol).await
    }

    /// Ambil n bar terakhir dari sumber tertentu
    pub async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let query = CandleQuery::new(symbol.clone(), timeframe, source).with_limit(limit);
        self.query_candles(&query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    struct DummyAdapter {
        src: MarketDataSource,
    }

    #[async_trait]
    impl MarketDataPort for DummyAdapter {
        fn source(&self) -> MarketDataSource {
            self.src
        }

        async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
            Ok(Tick {
                symbol: symbol.clone(),
                timestamp: Utc::now(),
                source: self.src,
                bid: dec!(1.0),
                ask: dec!(1.0001),
            })
        }

        async fn get_recent_candles(
            &self,
            symbol: &Symbol,
            timeframe: Timeframe,
            limit: usize,
        ) -> Result<Vec<Candle>, DomainError> {
            Ok(vec![
                Candle {
                    symbol: symbol.clone(),
                    timeframe,
                    timestamp: Utc::now(),
                    source: self.src,
                    open: dec!(1.0),
                    high: dec!(1.1),
                    low: dec!(0.9),
                    close: dec!(1.05),
                    volume: dec!(100.0),
                };
                limit
            ])
        }

        async fn get_historical_candles(
            &self,
            _symbol: &Symbol,
            _timeframe: Timeframe,
            _from: chrono::DateTime<Utc>,
            _to: chrono::DateTime<Utc>,
        ) -> Result<Vec<Candle>, DomainError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_router_service_routes_strictly_by_source() {
        let mut router = MarketDataRouterService::new();
        router.register(Arc::new(DummyAdapter {
            src: MarketDataSource::DukascopyEcn,
        }));
        router.register(Arc::new(DummyAdapter {
            src: MarketDataSource::MrgMetaTrader4,
        }));

        let sym = Symbol::from_symbol_str("EURUSD").unwrap();

        // 1. Query Dukascopy
        let dukas_query =
            CandleQuery::new(sym.clone(), Timeframe::H1, MarketDataSource::DukascopyEcn)
                .with_limit(5);
        let dukas_res = router.query_candles(&dukas_query).await.unwrap();
        assert_eq!(dukas_res.len(), 5);
        assert_eq!(dukas_res[0].source, MarketDataSource::DukascopyEcn);

        // 2. Query MRG MT4
        let mrg_query =
            CandleQuery::new(sym.clone(), Timeframe::H1, MarketDataSource::MrgMetaTrader4)
                .with_limit(3);
        let mrg_res = router.query_candles(&mrg_query).await.unwrap();
        assert_eq!(mrg_res.len(), 3);
        assert_eq!(mrg_res[0].source, MarketDataSource::MrgMetaTrader4);

        // 3. Query unconfigured Ctrader source fails cleanly
        let ctrader_query =
            CandleQuery::new(sym.clone(), Timeframe::H1, MarketDataSource::CtraderOpenApi);
        assert!(router.query_candles(&ctrader_query).await.is_err());
    }
}
