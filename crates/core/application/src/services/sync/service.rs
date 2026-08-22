use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

use domain::errors::DomainError;
use domain::models::{MarketDataSource, Symbol, Timeframe};
use domain::ports::{DeltaSyncPort, DeltaSyncReport, MarketDataPort, StoragePort};

/// Service untuk sinkronisasi delta (High-Watermark Incremental Data Ingestion)
pub struct DeltaSyncService {
    storage: Arc<dyn StoragePort>,
    market_data: Arc<dyn MarketDataPort>,
}

impl DeltaSyncService {
    pub fn new(storage: Arc<dyn StoragePort>, market_data: Arc<dyn MarketDataPort>) -> Self {
        Self {
            storage,
            market_data,
        }
    }

    fn timeframe_to_duration(timeframe: Timeframe) -> Duration {
        match timeframe {
            Timeframe::M1 => Duration::minutes(1),
            Timeframe::M5 => Duration::minutes(5),
            Timeframe::M15 => Duration::minutes(15),
            Timeframe::M30 => Duration::minutes(30),
            Timeframe::H1 => Duration::hours(1),
            Timeframe::H4 => Duration::hours(4),
            Timeframe::D1 => Duration::days(1),
            Timeframe::W1 => Duration::weeks(1),
        }
    }
}

#[async_trait]
impl DeltaSyncPort for DeltaSyncService {
    async fn sync_pair_delta(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<DeltaSyncReport, DomainError> {
        let start_time = Instant::now();

        // 1. Ambil High-Watermark (titik timestamp tertinggi yang sudah tersimpan di storage)
        let previous_watermark = self
            .storage
            .get_high_watermark(symbol, timeframe, source)
            .await?;

        let tf_dur = Self::timeframe_to_duration(timeframe);
        let now = Utc::now();

        let from_ts = match previous_watermark {
            Some(ts) => ts + Duration::seconds(1),
            None => now - Duration::days(365), // Default 1 tahun jika belum ada data sama sekali
        };

        // 2. Evaluasi apakah storage sudah up-to-date (belum ada candle baru yang CLOSED)
        if now <= from_ts + tf_dur {
            return Ok(DeltaSyncReport {
                symbol: symbol.clone(),
                timeframe,
                source,
                previous_watermark,
                new_watermark: previous_watermark,
                synced_bars_count: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
                is_up_to_date: true,
            });
        }

        info!(
            "🔄 Memulai Delta Sync {} {:?} (Source: {:?}) dari {} s/d {}",
            symbol.to_compact_string(),
            timeframe,
            source,
            from_ts,
            now
        );

        // 3. Tarik hanya delta candle baru dari Market Data Provider
        let fetched_candles = self
            .market_data
            .get_historical_candles(symbol, timeframe, from_ts, now)
            .await?;

        // 4. Filter strictly: Hanya candle dengan timestamp > from_ts & pastikan source tag terpasang
        let mut valid_deltas = Vec::with_capacity(fetched_candles.len());
        for mut c in fetched_candles {
            if c.timestamp >= from_ts {
                c.source = source;
                valid_deltas.push(c);
            }
        }

        let synced_count = valid_deltas.len();
        let new_watermark = if let Some(last_candle) = valid_deltas.last() {
            // 5. Simpan secara idempotent ke storage
            self.storage.save_candles(&valid_deltas).await?;
            Some(last_candle.timestamp)
        } else {
            previous_watermark
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "✅ Delta Sync {} {:?} Selesai: {} bar baru disimpan (Durasi: {} ms)",
            symbol.to_compact_string(),
            timeframe,
            synced_count,
            duration_ms
        );

        Ok(DeltaSyncReport {
            symbol: symbol.clone(),
            timeframe,
            source,
            previous_watermark,
            new_watermark,
            synced_bars_count: synced_count,
            duration_ms,
            is_up_to_date: synced_count == 0,
        })
    }

    async fn sync_all_pairs(
        &self,
        symbols: &[Symbol],
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<Vec<DeltaSyncReport>, DomainError> {
        let mut reports = Vec::with_capacity(symbols.len());
        for sym in symbols {
            match self.sync_pair_delta(sym, timeframe, source).await {
                Ok(report) => reports.push(report),
                Err(err) => {
                    warn!(
                        "⚠️ Gagal delta sync pair {}: {}",
                        sym.to_compact_string(),
                        err
                    );
                }
            }
        }
        Ok(reports)
    }
}
