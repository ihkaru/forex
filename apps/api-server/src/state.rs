use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, PolaNStrategy, Symbol, Tick, Timeframe};
use domain::ports::MarketDataPort;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RealHistoricalMarketAdapter {
    pub candles_map: HashMap<String, Vec<Candle>>,
}

#[derive(serde::Deserialize)]
struct RawCandleJson {
    timestamp: String,
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
        let pairs = ["EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD"];

        for p in &pairs {
            if let Some(sym) = Symbol::from_symbol_str(p) {
                let candles = Self::load_real_market_candles(&sym).unwrap_or_else(|e| {
                    eprintln!("⚠️ Gagal memuat data nyata untuk {}: {}", p, e);
                    Vec::new()
                });
                map.insert(sym.to_compact_string(), candles);
            }
        }

        Self { candles_map: map }
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
            let ts = DateTime::parse_from_rfc3339(&raw.timestamp)?.with_timezone(&Utc);
            let open = Decimal::from_str(&raw.open)?;
            let high = Decimal::from_str(&raw.high)?;
            let low = Decimal::from_str(&raw.low)?;
            let close = Decimal::from_str(&raw.close)?;
            let volume = Decimal::from_str(&raw.volume)?;

            candles.push(Candle {
                symbol: symbol.clone(),
                timeframe: Timeframe::H1,
                timestamp: ts,
                source: domain::models::MarketDataSource::DukascopyEcn,
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
        if let Some(candles) = self.candles_map.get(&symbol.to_compact_string()) {
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
        if let Some(candles) = self.candles_map.get(&symbol.to_compact_string()) {
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
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        if let Some(candles) = self.candles_map.get(&symbol.to_compact_string()) {
            return Ok(candles.clone());
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
}
