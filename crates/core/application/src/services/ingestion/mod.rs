use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, MarketDataSource, Symbol, TfPairSpec, Timeframe};
use domain::ports::ingestion::{
    IngestionRequest, IngestionResult, MarketIngestionPort, SymbolStatusDto,
};
use domain::ports::StoragePort;
use dukascopy_rs::DukascopyDownloader;
use rust_decimal::Decimal;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(serde::Serialize, serde::Deserialize)]
struct RawDiskCandle {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub timestamp: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub time: i64,
    pub source: MarketDataSource,
}

/// Service Baku Ingesti Data Pasar Historis ECN (Traders Family Compliant)
pub struct MarketIngestionService {
    downloader: Arc<DukascopyDownloader>,
    storage: Arc<dyn StoragePort>,
    data_dir: PathBuf,
}

impl MarketIngestionService {
    pub fn new(storage: Arc<dyn StoragePort>) -> Self {
        let base_path = Self::find_historical_data_dir();
        Self {
            downloader: Arc::new(DukascopyDownloader::new()),
            storage,
            data_dir: base_path,
        }
    }

    pub fn with_custom_dir(
        downloader: Arc<DukascopyDownloader>,
        storage: Arc<dyn StoragePort>,
        data_dir: PathBuf,
    ) -> Self {
        Self {
            downloader,
            storage,
            data_dir,
        }
    }

    fn find_historical_data_dir() -> PathBuf {
        let p1 = Path::new("data/historical");
        if p1.exists() {
            return p1.to_path_buf();
        }
        let p2 = Path::new("../../data/historical");
        if p2.exists() {
            return p2.to_path_buf();
        }
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let p3 = PathBuf::from(manifest).join("../../data/historical");
            if p3.exists() {
                return p3;
            }
        }
        PathBuf::from("data/historical")
    }

    fn load_disk_candles(&self, symbol: &Symbol) -> Vec<Candle> {
        let sym_str = symbol.to_compact_string();
        let file_path = self.data_dir.join(format!("{}_H1.json", sym_str));

        if !file_path.exists() {
            return Vec::new();
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let raw_candles: Vec<RawDiskCandle> = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        use std::str::FromStr;
        let mut candles = Vec::with_capacity(raw_candles.len());
        for raw in raw_candles {
            let ts = DateTime::from_timestamp(raw.time, 0).unwrap_or_else(Utc::now);
            if let (Ok(o), Ok(h), Ok(l), Ok(c), Ok(v)) = (
                Decimal::from_str(&raw.open),
                Decimal::from_str(&raw.high),
                Decimal::from_str(&raw.low),
                Decimal::from_str(&raw.close),
                Decimal::from_str(&raw.volume),
            ) {
                if o > Decimal::ZERO && h >= l {
                    candles.push(Candle {
                        symbol: symbol.clone(),
                        timeframe: Timeframe::H1,
                        timestamp: ts,
                        source: raw.source,
                        open: o,
                        high: h,
                        low: l,
                        close: c,
                        volume: v,
                    });
                }
            }
        }

        candles
    }

    fn save_disk_candles(&self, symbol: &Symbol, candles: &[Candle]) -> Result<(), DomainError> {
        let _ = std::fs::create_dir_all(&self.data_dir);
        let sym_str = symbol.to_compact_string();
        let file_path = self.data_dir.join(format!("{}_H1.json", sym_str));

        let disk_records: Vec<RawDiskCandle> = candles
            .iter()
            .map(|c| RawDiskCandle {
                symbol: c.symbol.clone(),
                timeframe: c.timeframe,
                timestamp: c.timestamp.to_rfc3339(),
                open: c.open.to_string(),
                high: c.high.to_string(),
                low: c.low.to_string(),
                close: c.close.to_string(),
                volume: c.volume.to_string(),
                time: c.timestamp.timestamp(),
                source: c.source,
            })
            .collect();

        let json_str = serde_json::to_string_pretty(&disk_records).map_err(|e| {
            DomainError::AdapterError(format!("Gagal serialize candle disk: {}", e))
        })?;

        std::fs::write(&file_path, json_str).map_err(|e| {
            DomainError::AdapterError(format!("Gagal menulis file candle disk: {}", e))
        })?;

        Ok(())
    }
}

#[async_trait]
impl MarketIngestionPort for MarketIngestionService {
    async fn ingest_symbol(&self, req: IngestionRequest) -> Result<IngestionResult, DomainError> {
        info!(
            "Memulai pipeline ingesti data baku untuk {} ({}-{})",
            req.symbol, req.from_year, req.to_year
        );

        let spec = TfPairSpec::from_symbol(&req.symbol);

        // 1. Ambil candle yang sudah ada di disk
        let mut existing_candles = self.load_disk_candles(&req.symbol);
        let initial_count = existing_candles.len();

        // 2. Fetch data dari Dukascopy Bank SA
        let fetched_candles = self
            .downloader
            .fetch_candles_range(&req.symbol, req.from_year, req.to_year)
            .await?;

        info!(
            "Berhasil mengunduh {} bar baru dari Dukascopy untuk {}",
            fetched_candles.len(),
            req.symbol
        );

        // 3. Idempotent Merge & Deduplication
        existing_candles.extend(fetched_candles);
        existing_candles.sort_by_key(|c| c.timestamp);
        existing_candles.dedup_by_key(|c| c.timestamp);

        let final_count = existing_candles.len();
        let new_added = final_count.saturating_sub(initial_count);

        // 4. Simpan ke Disk Cache
        self.save_disk_candles(&req.symbol, &existing_candles)?;

        // 5. Simpan ke Storage Database Port
        if let Err(e) = self.storage.save_candles(&existing_candles).await {
            warn!("Peringatan: Gagal menyimpan ke storage adapter: {}", e);
        }

        let first_ts = existing_candles.first().map(|c| c.timestamp);
        let last_ts = existing_candles.last().map(|c| c.timestamp);

        let tier_num = match spec.tier {
            domain::models::PairTier::Tier1 => 1,
            domain::models::PairTier::Tier2 => 2,
            domain::models::PairTier::Tier3 => 3,
            domain::models::PairTier::Tier4 => 4,
        };

        Ok(IngestionResult {
            status: "SUCCESS".to_string(),
            symbol: req.symbol,
            timeframe: req.timeframe,
            total_candles: final_count,
            new_candles_added: new_added,
            first_timestamp: first_ts,
            last_timestamp: last_ts,
            tier: tier_num,
            multiplier: spec.value_multiplier,
            min_sl_tp_pips: spec.min_sl_tp_pips,
        })
    }

    async fn list_available_symbols(&self) -> Result<Vec<SymbolStatusDto>, DomainError> {
        let default_pairs = [
            "EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD", "XAUUSD", "USDCAD",
            "USDJPY", "EURJPY", "GBPJPY", "CADJPY", "AUDJPY", "NZDJPY", "CHFJPY", "EURNZD",
        ];

        let mut results = Vec::new();

        for p in &default_pairs {
            if let Some(sym) = Symbol::from_symbol_str(p) {
                let spec = TfPairSpec::from_symbol(&sym);
                let candles = self.load_disk_candles(&sym);
                let candle_count = candles.len();
                let first_ts = candles.first().map(|c| c.timestamp);
                let last_ts = candles.last().map(|c| c.timestamp);
                let is_available = candle_count > 0;

                let tier_num = match spec.tier {
                    domain::models::PairTier::Tier1 => 1,
                    domain::models::PairTier::Tier2 => 2,
                    domain::models::PairTier::Tier3 => 3,
                    domain::models::PairTier::Tier4 => 4,
                };

                results.push(SymbolStatusDto {
                    symbol: sym,
                    tier: tier_num,
                    multiplier: spec.value_multiplier,
                    pip_size: spec.pip_size,
                    min_sl_tp_pips: spec.min_sl_tp_pips,
                    max_sl_tp_pips: spec.max_sl_tp_pips,
                    candle_count,
                    first_timestamp: first_ts,
                    last_timestamp: last_ts,
                    is_available,
                });
            }
        }

        Ok(results)
    }
}
