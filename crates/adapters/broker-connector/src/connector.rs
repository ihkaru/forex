use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, CandleQuery, MarketDataSource, Symbol, Tick, Timeframe};
use domain::ports::MarketDataPort;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::messages::{Mt5SocketMessage, MAX_CANDLE_CAPACITY};
use crate::validator::DataIntegrityValidator;

pub type CandleBufferMap =
    Arc<RwLock<HashMap<(Symbol, Timeframe, MarketDataSource), VecDeque<Candle>>>>;

pub struct BrokerConnector {
    pub broker_name: String,
    latest_ticks: Arc<RwLock<HashMap<Symbol, Tick>>>,
    candle_buffer: CandleBufferMap,
    raw_symbol_map: Arc<RwLock<HashMap<Symbol, String>>>,
}

impl BrokerConnector {
    pub fn new(broker_name: impl Into<String>) -> Self {
        Self {
            broker_name: broker_name.into(),
            latest_ticks: Arc::new(RwLock::new(HashMap::new())),
            candle_buffer: Arc::new(RwLock::new(HashMap::new())),
            raw_symbol_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_raw_symbol(&self, symbol: &Symbol) -> Option<String> {
        let lock = self.raw_symbol_map.read().await;
        lock.get(symbol).cloned()
    }

    pub async fn clear_all_data(&self) -> usize {
        let mut buf_lock = self.candle_buffer.write().await;
        let count: usize = buf_lock.values().map(|v| v.len()).sum();
        buf_lock.clear();

        let mut ticks_lock = self.latest_ticks.write().await;
        ticks_lock.clear();

        let mut raw_lock = self.raw_symbol_map.write().await;
        raw_lock.clear();

        tracing::info!("🧹 BrokerConnector: Seluruh {} candle dan buffer tick berhasil di-reset", count);
        count
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
                    Symbol::from_symbol_str(&symbol).ok_or(DomainError::InvalidSymbol(symbol.clone()))?;

                // 1. Validasi Integritas Harga
                DataIntegrityValidator::validate_tick(bid, ask)?;

                // 2. Normalisasi Waktu ke UTC
                let utc_time = DataIntegrityValidator::normalize_to_utc(time_gmt);

                let src = source
                    .as_deref()
                    .and_then(|s| MarketDataSource::from_source_str(s).ok())
                    .unwrap_or(MarketDataSource::MrgDemoMt4);

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
                lock.insert(symbol_obj.clone(), tick);
                self.raw_symbol_map.write().await.insert(symbol_obj, symbol);
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
                    Symbol::from_symbol_str(&symbol).ok_or(DomainError::InvalidSymbol(symbol.clone()))?;
                self.raw_symbol_map.write().await.insert(symbol_obj.clone(), symbol.clone());
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
                    .and_then(|s| MarketDataSource::from_source_str(s).ok())
                    .unwrap_or(MarketDataSource::MrgDemoMt4);

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
    fn source(&self) -> MarketDataSource {
        MarketDataSource::MrgDemoMt4
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
            MarketDataSource::MrgDemoMt4,
            MarketDataSource::MrgRealMt4,
            MarketDataSource::MrgMetaTrader4,
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
        query: &CandleQuery,
    ) -> Result<Vec<Candle>, DomainError> {
        let lock = self.candle_buffer.read().await;
        let target_candles = if let Some(candles) =
            lock.get(&(query.symbol.clone(), query.timeframe, query.source))
        {
            candles
        } else if let Some(candles) = lock.get(&(
            query.symbol.clone(),
            query.timeframe,
            MarketDataSource::MrgDemoMt4,
        )) {
            candles
        } else if let Some(candles) = lock.get(&(
            query.symbol.clone(),
            query.timeframe,
            MarketDataSource::MrgRealMt4,
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
