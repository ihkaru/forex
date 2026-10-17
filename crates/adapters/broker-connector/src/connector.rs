use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, CandleQuery, MarketDataSource, Symbol, Tick, Timeframe};
use domain::ports::{BrokerSpecPort, MarketDataPort, StoragePort, TickStoragePort};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::messages::{Mt5SocketMessage, MAX_CANDLE_CAPACITY};
use crate::validator::DataIntegrityValidator;

pub type CandleBufferMap =
    Arc<RwLock<HashMap<(Symbol, Timeframe, MarketDataSource), VecDeque<Candle>>>>;

pub const MAX_TICK_CAPACITY: usize = 10_000;

pub struct BrokerConnector {
    pub broker_name: String,
    latest_ticks: Arc<RwLock<HashMap<Symbol, Tick>>>,
    tick_buffer: Arc<RwLock<HashMap<(Symbol, MarketDataSource), VecDeque<Tick>>>>,
    candle_buffer: CandleBufferMap,
    raw_symbol_map: Arc<RwLock<HashMap<Symbol, String>>>,
    broker_specs: Arc<RwLock<HashMap<Symbol, domain::models::BrokerSymbolSpec>>>,
    storage: Option<Arc<dyn StoragePort>>,
    tick_storage: Option<Arc<dyn TickStoragePort>>,
    broker_spec_storage: Option<Arc<dyn BrokerSpecPort>>,
}

impl BrokerConnector {
    pub fn new(broker_name: impl Into<String>) -> Self {
        Self {
            broker_name: broker_name.into(),
            latest_ticks: Arc::new(RwLock::new(HashMap::new())),
            tick_buffer: Arc::new(RwLock::new(HashMap::new())),
            candle_buffer: Arc::new(RwLock::new(HashMap::new())),
            raw_symbol_map: Arc::new(RwLock::new(HashMap::new())),
            broker_specs: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
            tick_storage: None,
            broker_spec_storage: None,
        }
    }

    pub fn with_storage(
        mut self,
        storage: Arc<dyn StoragePort>,
        tick_storage: Arc<dyn TickStoragePort>,
        broker_spec_storage: Arc<dyn BrokerSpecPort>,
    ) -> Self {
        self.storage = Some(storage);
        self.tick_storage = Some(tick_storage);
        self.broker_spec_storage = Some(broker_spec_storage);
        self
    }

    pub async fn get_raw_symbol(&self, symbol: &Symbol) -> Option<String> {
        let lock = self.raw_symbol_map.read().await;
        lock.get(symbol).cloned()
    }

    pub async fn get_broker_spec(&self, symbol: &Symbol) -> Option<domain::models::BrokerSymbolSpec> {
        let lock = self.broker_specs.read().await;
        lock.get(symbol).cloned()
    }

    pub async fn list_broker_specs(&self) -> Vec<domain::models::BrokerSymbolSpec> {
        let lock = self.broker_specs.read().await;
        lock.values().cloned().collect()
    }

    pub async fn clear_all_data(&self) -> usize {
        let mut buf_lock = self.candle_buffer.write().await;
        let count: usize = buf_lock.values().map(|v| v.len()).sum();
        buf_lock.clear();

        let mut ticks_lock = self.latest_ticks.write().await;
        ticks_lock.clear();

        let mut tick_buf_lock = self.tick_buffer.write().await;
        tick_buf_lock.clear();

        let mut raw_lock = self.raw_symbol_map.write().await;
        raw_lock.clear();

        let mut specs_lock = self.broker_specs.write().await;
        specs_lock.clear();

        tracing::info!("🧹 BrokerConnector: Seluruh {} candle, buffer tick, dan spec berhasil di-reset", count);
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
                spread_pts,
                volume,
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
                    spread_pts: Some(spread_pts),
                    volume,
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
                lock.insert(symbol_obj.clone(), tick.clone());
                drop(lock);

                // Buffer tick streaming ke dalam ring buffer terbatas untuk backtest presisi tinggi
                let mut tb_lock = self.tick_buffer.write().await;
                let t_deque = tb_lock.entry((symbol_obj.clone(), src)).or_default();
                t_deque.push_back(tick.clone());
                while t_deque.len() > MAX_TICK_CAPACITY {
                    t_deque.pop_front();
                }
                drop(tb_lock);

                if let Some(ref ts) = self.tick_storage {
                    if let Err(e) = ts.save_ticks(&[tick.clone()]).await {
                        tracing::warn!("⚠️ Gagal persist live tick ke storage: {}", e);
                    }
                }

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
                spread_pts,
                real_volume,
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
                    .unwrap_or(MarketDataSource::MrgRealMt4);

                let spread_dec = spread_pts.map(|pts| {
                    let point_mult = if symbol_obj.quote == "JPY" || symbol_obj.base.contains("XAU") {
                        rust_decimal_macros::dec!(0.01)
                    } else {
                        rust_decimal_macros::dec!(0.00001)
                    };
                    rust_decimal::Decimal::from(pts) * point_mult
                });

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
                    spread: spread_dec,
                    real_volume,
                };

                let mut lock = self.candle_buffer.write().await;
                let deque = lock.entry((symbol_obj, tf, src)).or_default();

                // Idempotent Upsert: Jika bar di timestamp yang sama sudah ada (current forming bar),
                // perbarui bar in-place. Jika bar baru, sisipkan secara terurut dan jaga bounded capacity.
                if let Some(pos) = deque.iter().position(|c| c.timestamp == utc_time) {
                    deque[pos] = candle.clone();
                } else if let Some(last) = deque.back() {
                    if utc_time > last.timestamp {
                        deque.push_back(candle.clone());
                    } else {
                        let insert_pos = deque
                            .iter()
                            .position(|c| c.timestamp > utc_time)
                            .unwrap_or(deque.len());
                        deque.insert(insert_pos, candle.clone());
                    }
                } else {
                    deque.push_back(candle.clone());
                }

                // Enforce bounded circular buffer limit O(1)
                while deque.len() > MAX_CANDLE_CAPACITY {
                    deque.pop_front();
                }

                if let Some(ref s) = self.storage {
                    if let Err(e) = s.save_candles(&[candle.clone()]).await {
                        tracing::warn!("⚠️ Gagal persist live candle ke storage: {}", e);
                    }
                }
            }
            Mt5SocketMessage::Spec {
                symbol,
                source: _,
                digits,
                point,
                tick_value,
                tick_size,
                contract_size,
                swap_long,
                swap_short,
                swap_rollover_3days,
                stop_level_pts,
                freeze_level_pts,
                min_lot,
                lot_step,
                max_lot,
                margin_required,
            } => {
                let symbol_obj =
                    Symbol::from_symbol_str(&symbol).ok_or(DomainError::InvalidSymbol(symbol.clone()))?;
                let spec = domain::models::BrokerSymbolSpec {
                    symbol: symbol_obj.clone(),
                    raw_ticker: symbol.clone(),
                    digits,
                    point,
                    tick_value,
                    tick_size,
                    contract_size,
                    swap_long,
                    swap_short,
                    swap_rollover_3days,
                    stop_level_pts,
                    freeze_level_pts,
                    min_lot,
                    lot_step,
                    max_lot,
                    margin_required,
                };
                self.broker_specs.write().await.insert(symbol_obj.clone(), spec.clone());
                self.raw_symbol_map.write().await.insert(symbol_obj, symbol);

                if let Some(ref bss) = self.broker_spec_storage {
                    if let Err(e) = bss.save_broker_spec(&spec).await {
                        tracing::warn!("⚠️ Gagal persist broker spec ke storage: {}", e);
                    }
                }

                tracing::info!("📋 Broker Symbol Spec tersimpan dari bridge");
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
        MarketDataSource::MrgRealMt4
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
            MarketDataSource::MrgRealMt4,
            MarketDataSource::MrgDemoMt4,
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
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let lock = self.candle_buffer.read().await;
        for src in [
            MarketDataSource::MrgRealMt4,
            MarketDataSource::MrgDemoMt4,
            MarketDataSource::MrgMetaTrader4,
        ] {
            if let Some(candles) = lock.get(&(symbol.clone(), timeframe, src)) {
                if !candles.is_empty() {
                    let filtered: Vec<Candle> = candles
                        .iter()
                        .filter(|c| c.timestamp >= from && c.timestamp <= to)
                        .cloned()
                        .collect();
                    if !filtered.is_empty() {
                        return Ok(filtered);
                    }
                    // Jika rentang dari/ke lebih luas atau di luar timestamp buffer tetapi candle tersedia,
                    // kembalikan seluruh lilin buffer terkini agar simulasi backtest tetap berjalan deterministik.
                    return Ok(candles.iter().cloned().collect());
                }
            }
        }

        if let Some(ref s) = self.storage {
            for src in [
                MarketDataSource::MrgRealMt4,
                MarketDataSource::MrgDemoMt4,
                MarketDataSource::MrgMetaTrader4,
            ] {
                if let Ok(res) = s.get_historical_candles_range(symbol, timeframe, src, from, to).await {
                    if !res.is_empty() {
                        return Ok(res);
                    }
                }
            }
        }

        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: Buffer candle kosong untuk simbol {} timeframe {:?}. Belum ada data bar yang dikirim oleh MT4 EA via TCP socket atau tersimpan di storage.",
            symbol, timeframe
        )))
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
            MarketDataSource::MrgRealMt4,
        )) {
            candles
        } else if let Some(candles) = lock.get(&(
            query.symbol.clone(),
            query.timeframe,
            MarketDataSource::MrgDemoMt4,
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

    async fn get_historical_ticks(
        &self,
        symbol: &Symbol,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DomainError> {
        let lock = self.tick_buffer.read().await;
        for src in [
            MarketDataSource::MrgRealMt4,
            MarketDataSource::MrgDemoMt4,
            MarketDataSource::MrgMetaTrader4,
        ] {
            if let Some(ticks) = lock.get(&(symbol.clone(), src)) {
                if !ticks.is_empty() {
                    let filtered: Vec<Tick> = ticks
                        .iter()
                        .filter(|t| t.timestamp >= from && t.timestamp <= to)
                        .cloned()
                        .collect();
                    return Ok(filtered);
                }
            }
        }
        if let Some(ref ts) = self.tick_storage {
            for src in [
                MarketDataSource::MrgRealMt4,
                MarketDataSource::MrgDemoMt4,
                MarketDataSource::MrgMetaTrader4,
            ] {
                if let Ok(res) = ts.get_ticks(symbol, src, from, to).await {
                    if !res.is_empty() {
                        return Ok(res);
                    }
                }
            }
        }
        Ok(vec![])
    }

    async fn get_broker_spec(&self, symbol: &Symbol) -> Option<domain::models::BrokerSymbolSpec> {
        let lock = self.broker_specs.read().await;
        if let Some(spec) = lock.get(symbol).cloned() {
            return Some(spec);
        }
        if let Some(ref bss) = self.broker_spec_storage {
            if let Ok(Some(spec)) = bss.get_broker_spec(symbol).await {
                return Some(spec);
            }
        }
        None
    }
}

#[async_trait]
impl domain::ports::BrokerSpecPort for BrokerConnector {
    async fn save_broker_spec(
        &self,
        spec: &domain::models::BrokerSymbolSpec,
    ) -> Result<(), DomainError> {
        self.broker_specs
            .write()
            .await
            .insert(spec.symbol.clone(), spec.clone());
        Ok(())
    }

    async fn get_broker_spec(
        &self,
        symbol: &Symbol,
    ) -> Result<Option<domain::models::BrokerSymbolSpec>, DomainError> {
        let lock = self.broker_specs.read().await;
        Ok(lock.get(symbol).cloned())
    }

    async fn list_broker_specs(
        &self,
    ) -> Result<Vec<domain::models::BrokerSymbolSpec>, DomainError> {
        let lock = self.broker_specs.read().await;
        Ok(lock.values().cloned().collect())
    }
}

