use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::models::{Candle, MarketDataSource, Symbol, TfPairSpec, Tick, Timeframe};
use domain::ports::MarketDataPort;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

/// Multi-Symbol Real Historical Market Feed untuk simulasi turnamen kuantitatif
pub struct HistoricalMarketFeed {
    pub candles: HashMap<String, Vec<Candle>>,
    pub spread_pips: Decimal,
}

#[async_trait]
impl MarketDataPort for HistoricalMarketFeed {
    fn source(&self) -> MarketDataSource {
        MarketDataSource::DukascopyEcn
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, domain::errors::DomainError> {
        let spec = TfPairSpec::from_symbol(symbol);
        let last_close = self
            .candles
            .get(&symbol.to_compact_string())
            .and_then(|c| c.last())
            .map(|c| c.close)
            .ok_or_else(|| {
                domain::errors::DomainError::AdapterError(format!(
                    "Tidak ada data candle untuk symbol {}",
                    symbol.to_compact_string()
                ))
            })?;

        Ok(Tick {
            symbol: symbol.clone(),
            timestamp: Utc::now(),
            source: MarketDataSource::SyntheticTest,
            bid: last_close,
            ask: last_close + (spec.pip_size * self.spread_pips),
        })
    }

    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        _timeframe: Timeframe,
        _limit: usize,
    ) -> Result<Vec<Candle>, domain::errors::DomainError> {
        self.candles
            .get(&symbol.to_compact_string())
            .cloned()
            .ok_or_else(|| {
                domain::errors::DomainError::AdapterError(format!(
                    "Data recent candle tidak ditemukan untuk symbol: {}",
                    symbol.to_compact_string()
                ))
            })
    }

    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        _timeframe: Timeframe,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, domain::errors::DomainError> {
        self.candles
            .get(&symbol.to_compact_string())
            .cloned()
            .ok_or_else(|| {
                domain::errors::DomainError::AdapterError(format!(
                    "Data historical candle tidak ditemukan untuk symbol: {}",
                    symbol.to_compact_string()
                ))
            })
    }
}

#[derive(Deserialize)]
struct RawCandleJson {
    #[serde(default)]
    time: Option<i64>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    source: Option<MarketDataSource>,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

/// Memuat data pasar historis 100% nyata dari disk cache
pub fn load_real_market_candles(symbol: &Symbol) -> anyhow::Result<Vec<Candle>> {
    let sym_str = symbol.to_compact_string();
    let file_path = format!("data/historical/{}_H1.json", sym_str);

    if !std::path::Path::new(&file_path).exists() {
        println!(
            "⚠️ File {} belum ditemukan. Menjalankan downloader data resmi Dukascopy...",
            file_path
        );
        let status = std::process::Command::new("node")
            .arg("scripts/fetch_dukascopy_historical.js")
            .status()?;
        if !status.success() {
            anyhow::bail!("Gagal mengunduh data pasar resmi Dukascopy");
        }
    }

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
        let source = raw.source.unwrap_or(MarketDataSource::DukascopyEcn);
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
