use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use domain::errors::DomainError;
use domain::models::{Candle, Symbol, Tick, Timeframe};
use domain::ports::MarketDataPort;

/// Kapasitas maksimal bounded circular buffer untuk lilin pasar (menjaga footprint memori konstan O(1))
pub const MAX_CANDLE_CAPACITY: usize = 500;

/// Pesan mentah yang dikirimkan oleh MQL4/MQL5 EA Socket Bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Mt5SocketMessage {
    #[serde(rename = "TICK")]
    Tick {
        symbol: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        server: Option<String>,
        bid: Decimal,
        ask: Decimal,
        spread_pts: u32,
        time_gmt: i64,
    },
    #[serde(rename = "BAR")]
    Bar {
        symbol: String,
        #[serde(default)]
        source: Option<String>,
        timeframe: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        time_gmt: i64,
    },
}

/// Validasi Integritas Data & Sanitizer
pub struct DataIntegrityValidator;

impl DataIntegrityValidator {
    /// Validasi integritas harga Tick (Anti-Spike & Valid Spread)
    pub fn validate_tick(bid: Decimal, ask: Decimal) -> Result<(), DomainError> {
        if bid <= Decimal::ZERO || ask <= Decimal::ZERO {
            return Err(DomainError::InvalidPrice("Harga harus > 0".to_string()));
        }
        if ask < bid {
            return Err(DomainError::InvalidPrice(
                "Integritas Rusak: Ask lebih kecil dari Bid (Negative Spread)".to_string(),
            ));
        }
        let spread = ask - bid;
        // Kalibrasi peringatan anomali spread: Forex > 100 pips (0.0100) atau Gold > 500 pips ($5.00)
        let is_anomaly = if bid > dec!(100.0) {
            spread > dec!(5.00)
        } else {
            spread > dec!(0.0100)
        };
        if is_anomaly {
            warn!(
                "⚠️ Peringatan Anomali Pasar: Spread melebar drastis ({})",
                spread
            );
        }
        Ok(())
    }

    /// Konversi timestamp broker ke standar waktu internasional UTC
    pub fn normalize_to_utc(timestamp_seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(timestamp_seconds, 0)
            .single()
            .unwrap_or_else(Utc::now)
    }
}

pub type CandleBufferMap =
    Arc<RwLock<HashMap<(Symbol, Timeframe, domain::models::MarketDataSource), VecDeque<Candle>>>>;

pub struct BrokerConnector {
    pub broker_name: String,
    latest_ticks: Arc<RwLock<HashMap<Symbol, Tick>>>,
    candle_buffer: CandleBufferMap,
}

impl BrokerConnector {
    pub fn new(broker_name: impl Into<String>) -> Self {
        Self {
            broker_name: broker_name.into(),
            latest_ticks: Arc::new(RwLock::new(HashMap::new())),
            candle_buffer: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Ingest data mentah dari socket MQL4/MQL5 EA dengan validasi integritas
    pub async fn ingest_socket_message(&self, msg: Mt5SocketMessage) -> Result<(), DomainError> {
        match msg {
            Mt5SocketMessage::Tick {
                symbol,
                source,
                bid,
                ask,
                time_gmt,
                ..
            } => {
                if symbol.to_lowercase().contains("mock")
                    || source
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains("mock")
                {
                    return Err(DomainError::ValidationError(
                        "FAIL-FAST: Ditolak! Terdeteksi payload atau identifier 'mock' pada socket tick".to_string(),
                    ));
                }

                let symbol_obj =
                    Symbol::from_symbol_str(&symbol).ok_or(DomainError::InvalidSymbol(symbol))?;

                // 1. Validasi Integritas Harga
                DataIntegrityValidator::validate_tick(bid, ask)?;

                // 2. Normalisasi Waktu ke UTC
                let utc_time = DataIntegrityValidator::normalize_to_utc(time_gmt);

                let src = source
                    .as_deref()
                    .and_then(|s| domain::models::MarketDataSource::from_source_str(s).ok())
                    .unwrap_or(domain::models::MarketDataSource::MrgDemoMt4);

                let tick = Tick {
                    symbol: symbol_obj.clone(),
                    timestamp: utc_time,
                    source: src,
                    bid,
                    ask,
                };

                let mut lock = self.latest_ticks.write().await;
                // Monotonic Timestamp Guard: Menolak paket out-of-order yang lebih usang
                if let Some(existing) = lock.get(&symbol_obj) {
                    if utc_time < existing.timestamp {
                        tracing::debug!(
                            "⏳ Out-of-order tick diabaikan untuk {} (pkt: {}, current: {})",
                            symbol_obj,
                            utc_time,
                            existing.timestamp
                        );
                        return Ok(());
                    }
                }
                lock.insert(symbol_obj, tick);
            }
            Mt5SocketMessage::Bar {
                symbol,
                source,
                timeframe,
                open,
                high,
                low,
                close,
                volume,
                time_gmt,
            } => {
                if symbol.to_lowercase().contains("mock")
                    || source
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains("mock")
                {
                    return Err(DomainError::ValidationError(
                        "FAIL-FAST: Ditolak! Terdeteksi payload atau identifier 'mock' pada socket bar".to_string(),
                    ));
                }

                let symbol_obj =
                    Symbol::from_symbol_str(&symbol).ok_or(DomainError::InvalidSymbol(symbol))?;
                let utc_time = DataIntegrityValidator::normalize_to_utc(time_gmt);
                let tf = match timeframe.to_uppercase().as_str() {
                    "H1" | "1H" | "60" => Timeframe::H1,
                    "H4" | "4H" | "240" => Timeframe::H4,
                    "M30" | "30" => Timeframe::M30,
                    "M5" | "5" => Timeframe::M5,
                    "M1" | "1" => Timeframe::M1,
                    "D1" | "1D" | "1440" => Timeframe::D1,
                    _ => Timeframe::M15,
                };

                let src = source
                    .as_deref()
                    .and_then(|s| domain::models::MarketDataSource::from_source_str(s).ok())
                    .unwrap_or(domain::models::MarketDataSource::MrgDemoMt4);

                let candle = Candle {
                    symbol: symbol_obj.clone(),
                    timeframe: tf,
                    timestamp: utc_time,
                    source: src,
                    open,
                    high,
                    low,
                    close,
                    volume,
                };

                let mut lock = self.candle_buffer.write().await;
                let deque = lock.entry((symbol_obj, tf, src)).or_default();

                // Idempotent Upsert: Jika bar di timestamp yang sama sudah ada (current forming bar),
                // perbarui bar in-place. Jika bar baru, sisipkan secara terurut dan jaga bounded capacity.
                if let Some(pos) = deque.iter().position(|c| c.timestamp == utc_time) {
                    deque[pos] = candle;
                } else if let Some(last) = deque.back() {
                    if utc_time > last.timestamp {
                        deque.push_back(candle);
                    } else {
                        let insert_pos = deque
                            .iter()
                            .position(|c| c.timestamp > utc_time)
                            .unwrap_or(deque.len());
                        deque.insert(insert_pos, candle);
                    }
                } else {
                    deque.push_back(candle);
                }

                // Enforce bounded circular buffer limit O(1)
                while deque.len() > MAX_CANDLE_CAPACITY {
                    deque.pop_front();
                }
            }
        }
        Ok(())
    }

    /// Menjalankan TCP Socket Server non-blocking untuk menerima stream dari MT4/MT5 EA
    pub fn start_tcp_listener(self: Arc<Self>, host: &str, port: u16) {
        let addr = format!("{}:{}", host, port);
        tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => {
                    tracing::info!("📡 Broker TCP Socket Bridge mendengarkan di {}", addr);
                    l
                }
                Err(e) => {
                    tracing::error!(
                        "❌ Gagal membuka TCP Socket di {}: {}. Live feed dinonaktifkan.",
                        addr,
                        e
                    );
                    return;
                }
            };

            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        tracing::info!("🔗 Koneksi baru dari MetaTrader EA: {}", peer_addr);
                        let self_clone = self.clone();
                        tokio::spawn(async move {
                            use tokio::io::AsyncBufReadExt;
                            let reader = tokio::io::BufReader::new(stream);
                            let mut lines = reader.lines();

                            while let Ok(Some(line)) = lines.next_line().await {
                                let trimmed = line.trim();
                                if trimmed.is_empty() {
                                    continue;
                                }
                                match serde_json::from_str::<Mt5SocketMessage>(trimmed) {
                                    Ok(msg) => {
                                        if let Err(e) = self_clone.ingest_socket_message(msg).await
                                        {
                                            tracing::warn!(
                                                "⚠️ Penolakan validasi data integritas socket: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "⚠️ Format JSON socket broker corrupt: {} | Payload: {}",
                                            e,
                                            trimmed
                                        );
                                    }
                                }
                            }
                            tracing::info!("🔌 Koneksi MetaTrader EA terputus: {}", peer_addr);
                        });
                    }
                    Err(e) => {
                        tracing::error!("❌ Error socket accept: {}", e);
                    }
                }
            }
        });
    }
}

#[async_trait]
impl MarketDataPort for BrokerConnector {
    fn source(&self) -> domain::models::MarketDataSource {
        domain::models::MarketDataSource::MrgDemoMt4
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        let lock = self.latest_ticks.read().await;
        if let Some(tick) = lock.get(symbol) {
            return Ok(tick.clone());
        }

        // FAIL-FAST: Dilarang fallback! Hanya terima tick streaming langsung dari socket MT4.
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: Buffer tick kosong untuk simbol {} dari broker socket. Belum ada stream harga live dari MT5 EA.",
            symbol
        )))
    }

    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let lock = self.candle_buffer.read().await;
        for src in [
            domain::models::MarketDataSource::MrgDemoMt4,
            domain::models::MarketDataSource::MrgRealMt4,
            domain::models::MarketDataSource::MrgMetaTrader4,
        ] {
            if let Some(candles) = lock.get(&(symbol.clone(), timeframe, src)) {
                if !candles.is_empty() {
                    let skip = candles.len().saturating_sub(limit);
                    let result: Vec<Candle> = candles.iter().skip(skip).cloned().collect();
                    return Ok(result);
                }
            }
        }

        // FAIL-FAST: Dilarang fallback! Hanya terima candle streaming langsung dari socket MT4.
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: Buffer candle kosong untuk simbol {} timeframe {:?}. Belum ada data bar yang dikirim oleh MT4 EA via TCP socket.",
            symbol, timeframe
        )))
    }

    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        self.get_recent_candles(symbol, timeframe, 500).await
    }

    async fn query_candles(
        &self,
        query: &domain::models::CandleQuery,
    ) -> Result<Vec<Candle>, DomainError> {
        let lock = self.candle_buffer.read().await;
        let target_candles = if let Some(candles) =
            lock.get(&(query.symbol.clone(), query.timeframe, query.source))
        {
            candles
        } else if let Some(candles) = lock.get(&(
            query.symbol.clone(),
            query.timeframe,
            domain::models::MarketDataSource::MrgDemoMt4,
        )) {
            candles
        } else if let Some(candles) = lock.get(&(
            query.symbol.clone(),
            query.timeframe,
            domain::models::MarketDataSource::MrgRealMt4,
        )) {
            candles
        } else {
            return Ok(vec![]);
        };

        let mut filtered: Vec<Candle> = target_candles
            .iter()
            .filter(|c| {
                if let Some(from) = query.from {
                    if c.timestamp < from {
                        return false;
                    }
                }
                if let Some(to) = query.to {
                    if c.timestamp > to {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        if let Some(limit) = query.limit {
            if filtered.len() > limit {
                filtered = filtered.split_off(filtered.len() - limit);
            }
        }

        Ok(filtered)
    }
}

// ==============================================================================
// 2. cTRADER OPEN API CONNECTOR (Headless Protobuf Stream / Zero GUI Dependency)
// ==============================================================================

pub struct CtraderOpenApiConnector {
    pub client_id: String,
    pub client_secret: String,
    pub environment: String, // "demo.ctraderapi.com" or "live.ctraderapi.com"
}

impl CtraderOpenApiConnector {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        is_live: bool,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            environment: if is_live {
                "live.ctraderapi.com".to_string()
            } else {
                "demo.ctraderapi.com".to_string()
            },
        }
    }
}

#[async_trait]
impl MarketDataPort for CtraderOpenApiConnector {
    fn source(&self) -> domain::models::MarketDataSource {
        domain::models::MarketDataSource::CtraderOpenApi
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        // FAIL-FAST: Dilarang fallback ke snapshot mock!
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: cTrader Open API stream belum terkoneksi untuk simbol {}. Menolak fake tick.",
            symbol
        )))
    }

    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        _limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        // FAIL-FAST: Dilarang membuat lilin sintetis palsu!
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: cTrader Open API belum menerima streaming candle untuk simbol {} timeframe {:?}. Menolak synthetic candles.",
            symbol, timeframe
        )))
    }

    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        self.get_recent_candles(symbol, timeframe, 500).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_spread_integrity_rejection() {
        let bid = dec!(1.08500);
        let ask = dec!(1.08490); // Ask < Bid (Rusak)

        let result = DataIntegrityValidator::validate_tick(bid, ask);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_spread_acceptance() {
        let bid = dec!(1.08500);
        let ask = dec!(1.08515); // Normal spread 1.5 pips

        let result = DataIntegrityValidator::validate_tick(bid, ask);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ingest_socket_tick_message() {
        let connector = BrokerConnector::new("MetaTrader5-Socket");
        let msg = Mt5SocketMessage::Tick {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            server: Some("MaxrichGroup-Demo".to_string()),
            bid: dec!(1.08500),
            ask: dec!(1.08515),
            spread_pts: 15,
            time_gmt: 1720000000,
        };

        connector.ingest_socket_message(msg).await.unwrap();

        let symbol = Symbol::new("EUR", "USD");
        let tick = connector.get_latest_tick(&symbol).await.unwrap();
        assert_eq!(tick.bid, dec!(1.08500));
        assert_eq!(tick.ask, dec!(1.08515));
    }

    #[tokio::test]
    async fn test_monotonic_tick_rejection() {
        let connector = BrokerConnector::new("MetaTrader5-Socket");
        let symbol = Symbol::new("EUR", "USD");

        // Tick awal pada t = 1000
        connector
            .ingest_socket_message(Mt5SocketMessage::Tick {
                symbol: "EURUSD".to_string(),
                source: Some("MrgDemoMt4".to_string()),
                server: None,
                bid: dec!(1.08500),
                ask: dec!(1.08515),
                spread_pts: 15,
                time_gmt: 1000,
            })
            .await
            .unwrap();

        // Tick baru pada t = 1010
        connector
            .ingest_socket_message(Mt5SocketMessage::Tick {
                symbol: "EURUSD".to_string(),
                source: Some("MrgDemoMt4".to_string()),
                server: None,
                bid: dec!(1.08550),
                ask: dec!(1.08565),
                spread_pts: 15,
                time_gmt: 1010,
            })
            .await
            .unwrap();

        let tick = connector.get_latest_tick(&symbol).await.unwrap();
        assert_eq!(tick.bid, dec!(1.08550));

        // Out-of-order tick pada t = 1005 (lebih usang dari 1010, akibat TCP jitter)
        connector
            .ingest_socket_message(Mt5SocketMessage::Tick {
                symbol: "EURUSD".to_string(),
                source: Some("MrgDemoMt4".to_string()),
                server: None,
                bid: dec!(1.08400),
                ask: dec!(1.08415),
                spread_pts: 15,
                time_gmt: 1005,
            })
            .await
            .unwrap();

        // State harus tetap dipertahankan pada data terkini (t = 1010)
        let tick_after = connector.get_latest_tick(&symbol).await.unwrap();
        assert_eq!(tick_after.bid, dec!(1.08550));
    }

    #[tokio::test]
    async fn test_candle_buffer_upsert_and_bounded_capacity() {
        let connector = BrokerConnector::new("MetaTrader5-Socket");
        let symbol = Symbol::new("EUR", "USD");

        // 1. Ingest bar pertama t = 3600
        connector
            .ingest_socket_message(Mt5SocketMessage::Bar {
                symbol: "EURUSD".to_string(),
                source: Some("MrgDemoMt4".to_string()),
                timeframe: "H1".to_string(),
                open: dec!(1.08000),
                high: dec!(1.08200),
                low: dec!(1.07900),
                close: dec!(1.08100),
                volume: dec!(100),
                time_gmt: 3600,
            })
            .await
            .unwrap();

        let candles = connector
            .get_recent_candles(&symbol, Timeframe::H1, 10)
            .await
            .unwrap();
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].close, dec!(1.08100));

        // 2. Ingest update bar berjalan di timestamp yang sama t = 3600 (forming bar tick update)
        connector
            .ingest_socket_message(Mt5SocketMessage::Bar {
                symbol: "EURUSD".to_string(),
                source: Some("MrgDemoMt4".to_string()),
                timeframe: "H1".to_string(),
                open: dec!(1.08000),
                high: dec!(1.08300), // new high
                low: dec!(1.07900),
                close: dec!(1.08250), // new close
                volume: dec!(150),
                time_gmt: 3600,
            })
            .await
            .unwrap();

        let candles_updated = connector
            .get_recent_candles(&symbol, Timeframe::H1, 10)
            .await
            .unwrap();
        assert_eq!(
            candles_updated.len(),
            1,
            "Idempotent upsert harus mencegah duplikasi bar di timestamp yang sama"
        );
        assert_eq!(candles_updated[0].high, dec!(1.08300));
        assert_eq!(candles_updated[0].close, dec!(1.08250));

        // 3. Uji bounded capacity: Kirim 550 bar berurutan
        for i in 1..=550 {
            connector
                .ingest_socket_message(Mt5SocketMessage::Bar {
                    symbol: "EURUSD".to_string(),
                    source: Some("MrgDemoMt4".to_string()),
                    timeframe: "H1".to_string(),
                    open: dec!(1.08000),
                    high: dec!(1.08300),
                    low: dec!(1.07900),
                    close: dec!(1.08250),
                    volume: dec!(100),
                    time_gmt: 3600 + (i * 3600),
                })
                .await
                .unwrap();
        }

        let all_stored = connector
            .get_recent_candles(&symbol, Timeframe::H1, 1000)
            .await
            .unwrap();
        assert_eq!(
            all_stored.len(),
            MAX_CANDLE_CAPACITY,
            "Circular buffer harus dibatasi tepat di MAX_CANDLE_CAPACITY ({})",
            MAX_CANDLE_CAPACITY
        );
    }
}
