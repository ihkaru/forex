use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{
    BrokerSymbolSpec, Candle, MarketDataSource, Order, Signal, SignalAction, SignalStatus, Symbol,
    Tick, Timeframe,
};
use domain::ports::{BrokerSpecPort, StoragePort, TickStoragePort};
use rust_decimal::Decimal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

/// Embedded High-Performance SQLite Storage Engine
/// Mendukung mode WAL, batch chunked upsert berkecepatan tinggi, dan agnostik konsumen.
#[derive(Clone)]
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn connect(path: &str) -> Result<Self, DomainError> {
        info!("🔌 Menginisialisasi SQLite Storage di: {}", path);

        let options = if path == ":memory:" || path.starts_with("sqlite::memory:") {
            SqliteConnectOptions::from_str("sqlite::memory:")
                .map_err(|e| DomainError::AdapterError(format!("Invalid SQLite memory URI: {}", e)))?
                .create_if_missing(true)
        } else {
            // Pastikan direktori parent tersedia
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            SqliteConnectOptions::from_str(path)
                .map_err(|e| DomainError::AdapterError(format!("Invalid SQLite path: {}", e)))?
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await
            .map_err(|e| DomainError::AdapterError(format!("Gagal membuka SQLite pool: {}", e)))?;

        let storage = Self { pool };
        storage.init_schema().await?;
        Ok(storage)
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<(), DomainError> {
        let schema = r#"
            CREATE TABLE IF NOT EXISTS candles (
                symbol TEXT NOT NULL,
                timeframe TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                source TEXT NOT NULL,
                open TEXT NOT NULL,
                high TEXT NOT NULL,
                low TEXT NOT NULL,
                close TEXT NOT NULL,
                volume TEXT NOT NULL,
                spread TEXT,
                real_volume TEXT,
                PRIMARY KEY (symbol, timeframe, timestamp, source)
            );

            CREATE INDEX IF NOT EXISTS idx_candles_lookup 
            ON candles (symbol, timeframe, source, timestamp DESC);

            CREATE TABLE IF NOT EXISTS ticks (
                symbol TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                source TEXT NOT NULL,
                bid TEXT NOT NULL,
                ask TEXT NOT NULL,
                spread_pts INTEGER,
                volume TEXT NOT NULL,
                PRIMARY KEY (symbol, source, timestamp)
            );

            CREATE INDEX IF NOT EXISTS idx_ticks_lookup 
            ON ticks (symbol, source, timestamp DESC);

            CREATE TABLE IF NOT EXISTS broker_symbol_specs (
                symbol TEXT PRIMARY KEY,
                raw_ticker TEXT NOT NULL,
                digits INTEGER NOT NULL,
                point TEXT NOT NULL,
                tick_value TEXT NOT NULL,
                tick_size TEXT NOT NULL,
                contract_size TEXT NOT NULL,
                swap_long TEXT NOT NULL,
                swap_short TEXT NOT NULL,
                swap_rollover_3days INTEGER NOT NULL DEFAULT 3,
                stop_level_pts INTEGER NOT NULL DEFAULT 0,
                freeze_level_pts INTEGER NOT NULL DEFAULT 0,
                min_lot TEXT NOT NULL DEFAULT '0.01',
                lot_step TEXT NOT NULL DEFAULT '0.01',
                max_lot TEXT NOT NULL DEFAULT '100.0',
                margin_required TEXT NOT NULL DEFAULT '1000.0',
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS signals (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                action TEXT NOT NULL,
                timeframe TEXT NOT NULL,
                entry_price TEXT NOT NULL,
                stop_loss TEXT NOT NULL,
                take_profit_1 TEXT NOT NULL,
                take_profit_2 TEXT,
                take_profit_3 TEXT,
                risk_reward_ratio TEXT NOT NULL,
                confidence_score REAL NOT NULL,
                strategy_name TEXT NOT NULL,
                rationale TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'Active',
                created_at TEXT NOT NULL,
                expires_at TEXT
            );

            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                action TEXT NOT NULL,
                volume_lots TEXT NOT NULL,
                open_price TEXT NOT NULL,
                current_price TEXT NOT NULL,
                stop_loss TEXT NOT NULL,
                take_profit TEXT NOT NULL,
                open_time TEXT NOT NULL,
                close_time TEXT,
                realized_pnl TEXT
            );
        "#;

        sqlx::raw_sql(schema)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("Gagal migrasi SQLite schema: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl StoragePort for SqliteStorage {
    async fn save_signal(&self, signal: &Signal) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO signals (
                id, symbol, action, timeframe, entry_price, stop_loss,
                take_profit_1, take_profit_2, take_profit_3, risk_reward_ratio,
                confidence_score, strategy_name, rationale, status, created_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET status = excluded.status;
        "#;

        sqlx::query(query)
            .bind(signal.id.to_string())
            .bind(signal.symbol.to_compact_string())
            .bind(format!("{:?}", signal.action))
            .bind(format!("{:?}", signal.timeframe))
            .bind(signal.entry_price.to_string())
            .bind(signal.stop_loss.to_string())
            .bind(signal.take_profit_1.to_string())
            .bind(signal.take_profit_2.map(|p| p.to_string()))
            .bind(signal.take_profit_3.map(|p| p.to_string()))
            .bind(signal.risk_reward_ratio.to_string())
            .bind(signal.confidence_score)
            .bind(&signal.strategy_name)
            .bind(&signal.rationale)
            .bind(format!("{:?}", signal.status))
            .bind(signal.created_at.to_rfc3339())
            .bind(signal.expires_at.map(|e| e.to_rfc3339()))
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL save_signal: {}", e)))?;

        Ok(())
    }

    async fn get_signal(&self, id: Uuid) -> Result<Option<Signal>, DomainError> {
        let query = r#"
            SELECT id, symbol, action, timeframe, entry_price, stop_loss,
                   take_profit_1, take_profit_2, take_profit_3, risk_reward_ratio,
                   confidence_score, strategy_name, rationale, status, created_at, expires_at
            FROM signals WHERE id = ?
        "#;

        let row = sqlx::query(query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_signal: {}", e)))?;

        match row {
            Some(r) => {
                let sym_str: String = r.get("symbol");
                let symbol = Symbol::from_symbol_str(&sym_str).ok_or_else(|| {
                    DomainError::AdapterError(format!("Simbol corrupt: {}", sym_str))
                })?;
                let entry_str: String = r.get("entry_price");
                let sl_str: String = r.get("stop_loss");
                let tp1_str: String = r.get("take_profit_1");
                let tp2_str: Option<String> = r.get("take_profit_2");
                let tp3_str: Option<String> = r.get("take_profit_3");
                let rr_str: String = r.get("risk_reward_ratio");
                let created_str: String = r.get("created_at");
                let expires_str: Option<String> = r.get("expires_at");

                Ok(Some(Signal {
                    id,
                    symbol,
                    action: SignalAction::Buy,
                    timeframe: Timeframe::H1,
                    entry_price: Decimal::from_str(&entry_str).unwrap_or_default(),
                    stop_loss: Decimal::from_str(&sl_str).unwrap_or_default(),
                    take_profit_1: Decimal::from_str(&tp1_str).unwrap_or_default(),
                    take_profit_2: tp2_str.and_then(|s| Decimal::from_str(&s).ok()),
                    take_profit_3: tp3_str.and_then(|s| Decimal::from_str(&s).ok()),
                    risk_reward_ratio: Decimal::from_str(&rr_str).unwrap_or_default(),
                    confidence_score: r.get("confidence_score"),
                    strategy_name: r.get("strategy_name"),
                    rationale: r.get("rationale"),
                    status: SignalStatus::Active,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    expires_at: expires_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_active_signals(&self) -> Result<Vec<Signal>, DomainError> {
        let query = "SELECT id FROM signals WHERE status IN ('Active', 'Pending')";
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL active signals: {}", e)))?;

        let mut signals = Vec::new();
        for r in rows {
            let id_str: String = r.get("id");
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(sig) = self.get_signal(id).await? {
                    signals.push(sig);
                }
            }
        }
        Ok(signals)
    }

    async fn save_candles(&self, candles: &[Candle]) -> Result<(), DomainError> {
        if candles.is_empty() {
            return Ok(());
        }

        // Chunked batch transaction O(1) disk sync
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL begin tx: {}", e)))?;

        let query = r#"
            INSERT INTO candles (
                symbol, timeframe, timestamp, source, open, high, low, close, volume, spread, real_volume
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (symbol, timeframe, timestamp, source) DO UPDATE SET
                close = excluded.close,
                volume = excluded.volume,
                spread = coalesce(excluded.spread, candles.spread),
                real_volume = coalesce(excluded.real_volume, candles.real_volume);
        "#;

        for c in candles {
            sqlx::query(query)
                .bind(c.symbol.to_compact_string())
                .bind(format!("{:?}", c.timeframe))
                .bind(c.timestamp.to_rfc3339())
                .bind(format!("{:?}", c.source))
                .bind(c.open.to_string())
                .bind(c.high.to_string())
                .bind(c.low.to_string())
                .bind(c.close.to_string())
                .bind(c.volume.to_string())
                .bind(c.spread.map(|s| s.to_string()))
                .bind(c.real_volume.map(|rv| rv.to_string()))
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::AdapterError(format!("SQL batch candle: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL commit candles: {}", e)))?;

        Ok(())
    }

    async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let query = r#"
            SELECT symbol, timeframe, timestamp, source, open, high, low, close, volume, spread, real_volume
            FROM candles
            WHERE symbol = ? AND timeframe = ?
            ORDER BY timestamp DESC
            LIMIT ?;
        "#;

        let rows = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .bind(format!("{:?}", timeframe))
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_candles: {}", e)))?;

        let mut candles = Vec::with_capacity(rows.len());
        for r in rows {
            candles.push(Self::row_to_candle(&r)?);
        }
        candles.reverse();
        Ok(candles)
    }

    async fn get_high_watermark(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
    ) -> Result<Option<DateTime<Utc>>, DomainError> {
        let query = r#"
            SELECT MAX(timestamp) as max_ts
            FROM candles
            WHERE symbol = ? AND timeframe = ? AND source = ?;
        "#;

        let row = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .bind(format!("{:?}", timeframe))
            .bind(format!("{:?}", source))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL high watermark: {}", e)))?;

        if let Some(r) = row {
            if let Ok(ts_str) = r.try_get::<String, _>("max_ts") {
                if let Ok(parsed) = DateTime::parse_from_rfc3339(&ts_str) {
                    return Ok(Some(parsed.with_timezone(&Utc)));
                }
            }
        }
        Ok(None)
    }

    async fn get_historical_candles_range(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: MarketDataSource,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let query = r#"
            SELECT symbol, timeframe, timestamp, source, open, high, low, close, volume, spread, real_volume
            FROM candles
            WHERE symbol = ? AND timeframe = ? AND source = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp ASC;
        "#;

        let rows = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .bind(format!("{:?}", timeframe))
            .bind(format!("{:?}", source))
            .bind(from.to_rfc3339())
            .bind(to.to_rfc3339())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL historical candles range: {}", e)))?;

        let mut candles = Vec::with_capacity(rows.len());
        for r in rows {
            candles.push(Self::row_to_candle(&r)?);
        }
        Ok(candles)
    }

    async fn save_order(&self, order: &Order) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO orders (
                id, symbol, action, volume_lots, open_price, current_price,
                stop_loss, take_profit, open_time, close_time, realized_pnl
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET
                current_price = excluded.current_price,
                close_time = excluded.close_time,
                realized_pnl = excluded.realized_pnl;
        "#;

        sqlx::query(query)
            .bind(order.id.to_string())
            .bind(order.symbol.to_compact_string())
            .bind(format!("{:?}", order.action))
            .bind(order.volume_lots.to_string())
            .bind(order.open_price.to_string())
            .bind(order.current_price.to_string())
            .bind(order.stop_loss.to_string())
            .bind(order.take_profit.to_string())
            .bind(order.open_time.to_rfc3339())
            .bind(order.close_time.map(|ct| ct.to_rfc3339()))
            .bind(order.realized_pnl.map(|p| p.to_string()))
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL save_order: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl TickStoragePort for SqliteStorage {
    async fn save_ticks(&self, ticks: &[Tick]) -> Result<(), DomainError> {
        if ticks.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL begin tick tx: {}", e)))?;

        let query = r#"
            INSERT INTO ticks (symbol, timestamp, source, bid, ask, spread_pts, volume)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (symbol, source, timestamp) DO NOTHING;
        "#;

        for t in ticks {
            sqlx::query(query)
                .bind(t.symbol.to_compact_string())
                .bind(t.timestamp.to_rfc3339())
                .bind(format!("{:?}", t.source))
                .bind(t.bid.to_string())
                .bind(t.ask.to_string())
                .bind(t.spread_pts.map(|s| s as i64))
                .bind(t.volume.map(|v| v.to_string()))
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::AdapterError(format!("SQL batch tick: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL commit ticks: {}", e)))?;

        Ok(())
    }

    async fn get_ticks(
        &self,
        symbol: &Symbol,
        source: MarketDataSource,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DomainError> {
        let query = r#"
            SELECT symbol, timestamp, source, bid, ask, spread_pts, volume
            FROM ticks
            WHERE symbol = ? AND source = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp ASC;
        "#;

        let rows = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .bind(format!("{:?}", source))
            .bind(from.to_rfc3339())
            .bind(to.to_rfc3339())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_ticks: {}", e)))?;

        let mut ticks = Vec::with_capacity(rows.len());
        for r in rows {
            let ts_str: String = r.get("timestamp");
            let ts = DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let bid_str: String = r.get("bid");
            let ask_str: String = r.get("ask");
            let spread: Option<i64> = r.get("spread_pts");
            let vol_str: Option<String> = r.get("volume");

            ticks.push(Tick {
                symbol: symbol.clone(),
                timestamp: ts,
                source,
                bid: Decimal::from_str(&bid_str).unwrap_or_default(),
                ask: Decimal::from_str(&ask_str).unwrap_or_default(),
                spread_pts: spread.map(|s| s as u32),
                volume: vol_str.and_then(|s| Decimal::from_str(&s).ok()),
            });
        }
        Ok(ticks)
    }

    async fn get_tick_high_watermark(
        &self,
        symbol: &Symbol,
        source: MarketDataSource,
    ) -> Result<Option<DateTime<Utc>>, DomainError> {
        let query = "SELECT MAX(timestamp) as max_ts FROM ticks WHERE symbol = ? AND source = ?";
        let row = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .bind(format!("{:?}", source))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL tick high watermark: {}", e)))?;

        if let Some(r) = row {
            if let Ok(ts_str) = r.try_get::<String, _>("max_ts") {
                if let Ok(parsed) = DateTime::parse_from_rfc3339(&ts_str) {
                    return Ok(Some(parsed.with_timezone(&Utc)));
                }
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl BrokerSpecPort for SqliteStorage {
    async fn save_broker_spec(&self, spec: &BrokerSymbolSpec) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO broker_symbol_specs (
                symbol, raw_ticker, digits, point, tick_value, tick_size, contract_size,
                swap_long, swap_short, swap_rollover_3days, stop_level_pts, freeze_level_pts,
                min_lot, lot_step, max_lot, margin_required, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (symbol) DO UPDATE SET
                raw_ticker = excluded.raw_ticker,
                digits = excluded.digits,
                point = excluded.point,
                tick_value = excluded.tick_value,
                tick_size = excluded.tick_size,
                contract_size = excluded.contract_size,
                swap_long = excluded.swap_long,
                swap_short = excluded.swap_short,
                swap_rollover_3days = excluded.swap_rollover_3days,
                stop_level_pts = excluded.stop_level_pts,
                freeze_level_pts = excluded.freeze_level_pts,
                min_lot = excluded.min_lot,
                lot_step = excluded.lot_step,
                max_lot = excluded.max_lot,
                margin_required = excluded.margin_required,
                updated_at = excluded.updated_at;
        "#;

        sqlx::query(query)
            .bind(spec.symbol.to_compact_string())
            .bind(&spec.raw_ticker)
            .bind(spec.digits as i64)
            .bind(spec.point.to_string())
            .bind(spec.tick_value.to_string())
            .bind(spec.tick_size.to_string())
            .bind(spec.contract_size.to_string())
            .bind(spec.swap_long.to_string())
            .bind(spec.swap_short.to_string())
            .bind(spec.swap_rollover_3days as i64)
            .bind(spec.stop_level_pts as i64)
            .bind(spec.freeze_level_pts as i64)
            .bind(spec.min_lot.to_string())
            .bind(spec.lot_step.to_string())
            .bind(spec.max_lot.to_string())
            .bind(spec.margin_required.to_string())
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL save_broker_spec: {}", e)))?;

        Ok(())
    }

    async fn get_broker_spec(&self, symbol: &Symbol) -> Result<Option<BrokerSymbolSpec>, DomainError> {
        let query = "SELECT * FROM broker_symbol_specs WHERE symbol = ?";
        let row = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_broker_spec: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::row_to_spec(&r)?)),
            None => Ok(None),
        }
    }

    async fn list_broker_specs(&self) -> Result<Vec<BrokerSymbolSpec>, DomainError> {
        let query = "SELECT * FROM broker_symbol_specs ORDER BY symbol ASC";
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL list_broker_specs: {}", e)))?;

        let mut specs = Vec::with_capacity(rows.len());
        for r in rows {
            specs.push(Self::row_to_spec(&r)?);
        }
        Ok(specs)
    }
}

impl SqliteStorage {
    fn row_to_candle(r: &sqlx::sqlite::SqliteRow) -> Result<Candle, DomainError> {
        let sym_str: String = r.get("symbol");
        let symbol = Symbol::from_symbol_str(&sym_str)
            .ok_or_else(|| DomainError::AdapterError(format!("Simbol corrupt: {}", sym_str)))?;
        let tf_str: String = r.get("timeframe");
        let timeframe = match tf_str.as_str() {
            "H1" => Timeframe::H1,
            "H4" => Timeframe::H4,
            "M30" => Timeframe::M30,
            "M15" => Timeframe::M15,
            "M5" => Timeframe::M5,
            "M1" => Timeframe::M1,
            "D1" => Timeframe::D1,
            _ => Timeframe::H1,
        };
        let ts_str: String = r.get("timestamp");
        let timestamp = DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let src_str: String = r.get("source");
        let source = MarketDataSource::from_source_str(&src_str)
            .unwrap_or(MarketDataSource::DukascopyEcn);

        let o_str: String = r.get("open");
        let h_str: String = r.get("high");
        let l_str: String = r.get("low");
        let c_str: String = r.get("close");
        let v_str: String = r.get("volume");
        let sp_str: Option<String> = r.get("spread");
        let rv_str: Option<String> = r.get("real_volume");

        Ok(Candle {
            symbol,
            timeframe,
            timestamp,
            source,
            open: Decimal::from_str(&o_str).unwrap_or_default(),
            high: Decimal::from_str(&h_str).unwrap_or_default(),
            low: Decimal::from_str(&l_str).unwrap_or_default(),
            close: Decimal::from_str(&c_str).unwrap_or_default(),
            volume: Decimal::from_str(&v_str).unwrap_or_default(),
            spread: sp_str.and_then(|s| Decimal::from_str(&s).ok()),
            real_volume: rv_str.and_then(|s| Decimal::from_str(&s).ok()),
        })
    }

    fn row_to_spec(r: &sqlx::sqlite::SqliteRow) -> Result<BrokerSymbolSpec, DomainError> {
        let sym_str: String = r.get("symbol");
        let symbol = Symbol::from_symbol_str(&sym_str)
            .ok_or_else(|| DomainError::AdapterError(format!("Simbol corrupt: {}", sym_str)))?;
        let raw_ticker: String = r.get("raw_ticker");
        let digits: i64 = r.get("digits");
        let point: String = r.get("point");
        let tick_value: String = r.get("tick_value");
        let tick_size: String = r.get("tick_size");
        let contract_size: String = r.get("contract_size");
        let swap_long: String = r.get("swap_long");
        let swap_short: String = r.get("swap_short");
        let swap_3d: i64 = r.get("swap_rollover_3days");
        let stop_lvl: i64 = r.get("stop_level_pts");
        let freeze_lvl: i64 = r.get("freeze_level_pts");
        let min_lot: String = r.get("min_lot");
        let lot_step: String = r.get("lot_step");
        let max_lot: String = r.get("max_lot");
        let margin_req: String = r.get("margin_required");

        Ok(BrokerSymbolSpec {
            symbol,
            raw_ticker,
            digits: digits as u32,
            point: Decimal::from_str(&point).unwrap_or_default(),
            tick_value: Decimal::from_str(&tick_value).unwrap_or_default(),
            tick_size: Decimal::from_str(&tick_size).unwrap_or_default(),
            contract_size: Decimal::from_str(&contract_size).unwrap_or_default(),
            swap_long: Decimal::from_str(&swap_long).unwrap_or_default(),
            swap_short: Decimal::from_str(&swap_short).unwrap_or_default(),
            swap_rollover_3days: swap_3d as u32,
            stop_level_pts: stop_lvl as u32,
            freeze_level_pts: freeze_lvl as u32,
            min_lot: Decimal::from_str(&min_lot).unwrap_or_default(),
            lot_step: Decimal::from_str(&lot_step).unwrap_or_default(),
            max_lot: Decimal::from_str(&max_lot).unwrap_or_default(),
            margin_required: Decimal::from_str(&margin_req).unwrap_or_default(),
        })
    }
}
