use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, MarketDataSource, Order, Signal, Symbol, Tick, Timeframe};
use domain::ports::{DeltaSyncPort, MarketDataPort, StoragePort};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::service::DeltaSyncService;

#[derive(Default, Clone)]
struct MockSyncStorage {
    candles: Arc<RwLock<Vec<Candle>>>,
}

#[async_trait]
impl StoragePort for MockSyncStorage {
    async fn save_signal(&self, _signal: &Signal) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_signal(&self, _id: Uuid) -> Result<Option<Signal>, DomainError> {
        Ok(None)
    }
    async fn get_active_signals(&self) -> Result<Vec<Signal>, DomainError> {
        Ok(vec![])
    }
    async fn save_candles(&self, new_candles: &[Candle]) -> Result<(), DomainError> {
        let mut lock = self.candles.write().await;
        for c in new_candles {
            // Idempotent upsert simulation
            if let Some(pos) = lock.iter().position(|x| {
                x.symbol == c.symbol
                    && x.timeframe == c.timeframe
                    && x.timestamp == c.timestamp
                    && x.source == c.source
            }) {
                lock[pos] = c.clone();
            } else {
                lock.push(c.clone());
            }
        }
        Ok(())
    }
    async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let lock = self.candles.read().await;
        let list: Vec<Candle> = lock
            .iter()
            .filter(|c| &c.symbol == symbol && c.timeframe == timeframe)
            .take(limit)
            .cloned()
            .collect();
        Ok(list)
    }
    async fn get_high_watermark(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<Option<DateTime<Utc>>, DomainError> {
        let lock = self.candles.read().await;
        let max_ts = lock
            .iter()
            .filter(|c| &c.symbol == symbol && c.timeframe == timeframe && c.source == source)
            .map(|c| c.timestamp)
            .max();
        Ok(max_ts)
    }
    async fn save_order(&self, _order: &Order) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockProvider {
    candles: HashMap<String, Vec<Candle>>,
}

#[async_trait]
impl MarketDataPort for MockProvider {
    fn source(&self) -> MarketDataSource {
        MarketDataSource::DukascopyEcn
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        Ok(Tick {
            symbol: symbol.clone(),
            timestamp: Utc::now(),
            source: MarketDataSource::DukascopyEcn,
            bid: dec!(1.0850),
            ask: dec!(1.0851),
        })
    }
    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        _timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let list = match self.candles.get(&symbol.to_compact_string()) {
            Some(v) => v.clone(),
            None => Vec::new(),
        };
        Ok(list.into_iter().take(limit).collect())
    }
    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        _timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let list = match self.candles.get(&symbol.to_compact_string()) {
            Some(v) => v.clone(),
            None => Vec::new(),
        };
        let filtered = list
            .into_iter()
            .filter(|c| c.timestamp >= from && c.timestamp <= to)
            .collect();
        Ok(filtered)
    }
}

#[tokio::test]
async fn test_delta_sync_fetches_only_new_bars_since_high_watermark() {
    let symbol = Symbol::new("EUR", "USD");
    let base_time = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    // 1. Inisialisasi storage awal dengan 10 bar (jam 00:00 s/d 09:00)
    let storage = Arc::new(MockSyncStorage::default());
    let mut initial_storage_candles = Vec::new();
    for i in 0..10 {
        initial_storage_candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i),
            source: MarketDataSource::DukascopyEcn,
            open: dec!(1.0850),
            high: dec!(1.0860),
            low: dec!(1.0840),
            close: dec!(1.0855),
            volume: dec!(1000.0),
        });
    }
    storage
        .save_candles(&initial_storage_candles)
        .await
        .unwrap();

    // High watermark awal harus jam 09:00
    let hw = storage
        .get_high_watermark(&symbol, Timeframe::H1, MarketDataSource::DukascopyEcn)
        .await
        .unwrap();
    assert_eq!(hw, Some(base_time + Duration::hours(9)));

    // 2. Provider memiliki 15 bar (jam 00:00 s/d 14:00) -> ada 5 bar delta baru!
    let mut provider_candles = Vec::new();
    for i in 0..15 {
        provider_candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i),
            source: MarketDataSource::DukascopyEcn,
            open: dec!(1.0850),
            high: dec!(1.0860),
            low: dec!(1.0840),
            close: dec!(1.0855),
            volume: dec!(1000.0),
        });
    }

    let mut map = HashMap::new();
    map.insert(symbol.to_compact_string(), provider_candles);
    let provider = Arc::new(MockProvider { candles: map });

    // 3. Jalankan DeltaSyncService
    let sync_service = DeltaSyncService::new(storage.clone(), provider);
    let report = sync_service
        .sync_pair_delta(&symbol, Timeframe::H1, MarketDataSource::DukascopyEcn)
        .await
        .unwrap();

    // 4. Verifikasi: Hanya 5 bar baru yang di-sync!
    assert_eq!(report.synced_bars_count, 5);
    assert_eq!(
        report.previous_watermark,
        Some(base_time + Duration::hours(9))
    );
    assert_eq!(report.new_watermark, Some(base_time + Duration::hours(14)));

    // Total bar di storage sekarang harus 15 bar
    let final_hw = storage
        .get_high_watermark(&symbol, Timeframe::H1, MarketDataSource::DukascopyEcn)
        .await
        .unwrap();
    assert_eq!(final_hw, Some(base_time + Duration::hours(14)));
}
