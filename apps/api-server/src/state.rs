use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, PolaNStrategy, Symbol, Tick, Timeframe};
use domain::ports::MarketDataPort;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;

use std::sync::RwLock;

pub struct RealHistoricalMarketAdapter {
    pub candles_map: RwLock<HashMap<String, Vec<Candle>>>,
}

#[derive(serde::Deserialize)]
struct RawCandleJson {
    #[serde(default)]
    time: Option<i64>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    source: Option<domain::models::MarketDataSource>,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

impl Default for RealHistoricalMarketAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RealHistoricalMarketAdapter {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        let default_pairs = [
            "EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD", "XAUUSD", "USDCAD",
            "USDJPY", "EURJPY", "GBPJPY",
        ];

        for p in &default_pairs {
            if let Some(sym) = Symbol::from_symbol_str(p) {
                if let Ok(candles) = Self::load_real_market_candles(&sym) {
                    if !candles.is_empty() {
                        map.insert(sym.to_compact_string(), candles);
                    }
                }
            }
        }

        Self {
            candles_map: RwLock::new(map),
        }
    }

    /// Reload data simbol tertentu ke memori secara live
    pub fn reload_symbol(&self, symbol: &Symbol) -> Result<usize, DomainError> {
        let candles = Self::load_real_market_candles(symbol)
            .map_err(|e| DomainError::AdapterError(e.to_string()))?;
        let count = candles.len();
        if let Ok(mut map) = self.candles_map.write() {
            map.insert(symbol.to_compact_string(), candles);
        }
        Ok(count)
    }

    /// Memuat data pasar historis 100% nyata dari disk cache
    pub fn load_real_market_candles(symbol: &Symbol) -> anyhow::Result<Vec<Candle>> {
        use std::str::FromStr;

        let sym_str = symbol.to_compact_string();
        let path1 = format!("data/historical/{}_H1.json", sym_str);
        let path2 = format!("../../data/historical/{}_H1.json", sym_str);

        let file_path = if std::path::Path::new(&path1).exists() {
            path1
        } else if std::path::Path::new(&path2).exists() {
            path2
        } else if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let p3 = format!("{}/../../data/historical/{}_H1.json", manifest, sym_str);
            if std::path::Path::new(&p3).exists() {
                p3
            } else {
                anyhow::bail!(
                    "File histori nyata {}_H1.json tidak ditemukan di disk",
                    sym_str
                );
            }
        } else {
            anyhow::bail!(
                "File histori nyata {}_H1.json tidak ditemukan di disk",
                sym_str
            );
        };

        let file_content = std::fs::read_to_string(&file_path)?;
        let raw_candles: Vec<RawCandleJson> = serde_json::from_str(&file_content)?;

        let mut candles = Vec::with_capacity(raw_candles.len());
        for raw in raw_candles {
            let ts = if let Some(epoch_sec) = raw.time {
                DateTime::from_timestamp(epoch_sec, 0).unwrap_or_else(Utc::now)
            } else if let Some(ref iso_str) = raw.timestamp {
                DateTime::parse_from_rfc3339(iso_str)?.with_timezone(&Utc)
            } else {
                Utc::now()
            };
            let source = raw
                .source
                .unwrap_or(domain::models::MarketDataSource::DukascopyEcn);
            let open = Decimal::from_str(&raw.open)?;
            let high = Decimal::from_str(&raw.high)?;
            let low = Decimal::from_str(&raw.low)?;
            let close = Decimal::from_str(&raw.close)?;
            let volume = Decimal::from_str(&raw.volume)?;

            candles.push(Candle {
                symbol: symbol.clone(),
                timeframe: Timeframe::H1,
                timestamp: ts,
                source,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        Ok(candles)
    }
}

#[async_trait]
impl MarketDataPort for RealHistoricalMarketAdapter {
    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        let map = self
            .candles_map
            .read()
            .map_err(|_| DomainError::AdapterError("Failed to acquire read lock".to_string()))?;

        if let Some(candles) = map.get(&symbol.to_compact_string()) {
            if let Some(last) = candles.last() {
                return Ok(Tick {
                    symbol: symbol.clone(),
                    source: last.source,
                    bid: last.close,
                    ask: last.close + dec!(0.00012),
                    timestamp: last.timestamp,
                });
            }
        }
        Err(DomainError::AdapterError(format!(
            "Pair {} tidak ditemukan",
            symbol
        )))
    }

    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        _timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let map = self
            .candles_map
            .read()
            .map_err(|_| DomainError::AdapterError("Failed to acquire read lock".to_string()))?;

        if let Some(candles) = map.get(&symbol.to_compact_string()) {
            let start = if candles.len() > limit {
                candles.len() - limit
            } else {
                0
            };
            return Ok(candles[start..].to_vec());
        }
        Err(DomainError::AdapterError(format!(
            "Pair {} tidak ditemukan",
            symbol
        )))
    }

    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        _timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let map = self
            .candles_map
            .read()
            .map_err(|_| DomainError::AdapterError("Failed to acquire read lock".to_string()))?;

        if let Some(candles) = map.get(&symbol.to_compact_string()) {
            let filtered: Vec<Candle> = candles
                .iter()
                .filter(|c| c.timestamp >= from && c.timestamp <= to)
                .cloned()
                .collect();
            return Ok(filtered);
        }
        Err(DomainError::AdapterError(format!(
            "Histori {} tidak ditemukan",
            symbol
        )))
    }
}

pub struct AppState {
    pub market_adapter: Arc<RealHistoricalMarketAdapter>,
    pub strategy: Arc<PolaNStrategy>,
    pub storage: Arc<storage_db::InMemoryStorage>,
    pub ingestion_service: Arc<application::services::MarketIngestionService>,
}
