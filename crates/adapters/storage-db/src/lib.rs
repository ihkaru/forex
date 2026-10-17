use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use domain::errors::DomainError;
use domain::models::{BrokerSymbolSpec, Candle, Order, Signal, SignalAction, SignalStatus, Symbol, Tick, Timeframe};
use domain::ports::{BrokerSpecPort, StoragePort, TickStoragePort};

pub mod sqlite;
pub use sqlite::SqliteStorage;

pub mod market_adapter;
pub use market_adapter::StorageMarketDataAdapter;

// ==============================================================================
// 1. IN-MEMORY STORAGE (Fast Prototyping & Unit Testing Double)
// ==============================================================================

#[derive(Default, Clone)]
pub struct InMemoryStorage {
    signals: Arc<RwLock<HashMap<Uuid, Signal>>>,
    candles: Arc<RwLock<Vec<Candle>>>,
    ticks: Arc<RwLock<Vec<Tick>>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    specs: Arc<RwLock<HashMap<Symbol, BrokerSymbolSpec>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl StoragePort for InMemoryStorage {
    async fn save_signal(&self, signal: &Signal) -> Result<(), DomainError> {
        let mut map = self.signals.write().await;
        map.insert(signal.id, signal.clone());
        Ok(())
    }

    async fn get_signal(&self, id: Uuid) -> Result<Option<Signal>, DomainError> {
        let map = self.signals.read().await;
        Ok(map.get(&id).cloned())
    }

    async fn get_active_signals(&self) -> Result<Vec<Signal>, DomainError> {
        let map = self.signals.read().await;
        Ok(map.values().filter(|s| s.is_open()).cloned().collect())
    }

    async fn save_candles(&self, new_candles: &[Candle]) -> Result<(), DomainError> {
        let mut list = self.candles.write().await;
        list.extend_from_slice(new_candles);
        Ok(())
    }

    async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let list = self.candles.read().await;
        let filtered: Vec<Candle> = list
            .iter()
            .filter(|c| &c.symbol == symbol && c.timeframe == timeframe)
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn get_high_watermark(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: domain::models::MarketDataSource,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError> {
        let list = self.candles.read().await;
        let max_ts = list
            .iter()
            .filter(|c| &c.symbol == symbol && c.timeframe == timeframe && c.source == source)
            .map(|c| c.timestamp)
            .max();
        Ok(max_ts)
    }

    async fn get_historical_candles_range(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: domain::models::MarketDataSource,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let list = self.candles.read().await;
        let filtered: Vec<Candle> = list
            .iter()
            .filter(|c| {
                &c.symbol == symbol
                    && c.timeframe == timeframe
                    && c.source == source
                    && c.timestamp >= from
                    && c.timestamp <= to
            })
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn get_total_candle_count(&self) -> Result<usize, DomainError> {
        let list = self.candles.read().await;
        Ok(list.len())
    }

    async fn save_order(&self, order: &Order) -> Result<(), DomainError> {
        let mut map = self.orders.write().await;
        map.insert(order.id, order.clone());
        Ok(())
    }
}

#[async_trait]
impl TickStoragePort for InMemoryStorage {
    async fn save_ticks(&self, new_ticks: &[Tick]) -> Result<(), DomainError> {
        let mut list = self.ticks.write().await;
        list.extend_from_slice(new_ticks);
        Ok(())
    }

    async fn get_ticks(
        &self,
        symbol: &Symbol,
        source: domain::models::MarketDataSource,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Tick>, DomainError> {
        let list = self.ticks.read().await;
        let filtered: Vec<Tick> = list
            .iter()
            .filter(|t| &t.symbol == symbol && t.source == source && t.timestamp >= from && t.timestamp <= to)
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn get_tick_high_watermark(
        &self,
        symbol: &Symbol,
        source: domain::models::MarketDataSource,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError> {
        let list = self.ticks.read().await;
        let max_ts = list
            .iter()
            .filter(|t| &t.symbol == symbol && t.source == source)
            .map(|t| t.timestamp)
            .max();
        Ok(max_ts)
    }
}

#[async_trait]
impl BrokerSpecPort for InMemoryStorage {
    async fn save_broker_spec(&self, spec: &BrokerSymbolSpec) -> Result<(), DomainError> {
        let mut map = self.specs.write().await;
        map.insert(spec.symbol.clone(), spec.clone());
        Ok(())
    }

    async fn get_broker_spec(&self, symbol: &Symbol) -> Result<Option<BrokerSymbolSpec>, DomainError> {
        let map = self.specs.read().await;
        Ok(map.get(symbol).cloned())
    }

    async fn list_broker_specs(&self) -> Result<Vec<BrokerSymbolSpec>, DomainError> {
        let map = self.specs.read().await;
        Ok(map.values().cloned().collect())
    }
}

// ==============================================================================
// 2. TIMESCALEDB / SQLX PERSISTENCE STORAGE (Production OLTP & Hypertables)
// ==============================================================================

pub struct SqlxStorage {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl SqlxStorage {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }

    pub async fn connect(database_url: &str) -> Result<Self, DomainError> {
        info!("Menghubungkan ke TimescaleDB / PostgreSQL database...");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await
            .map_err(|e| DomainError::AdapterError(format!("Gagal koneksi database: {}", e)))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl StoragePort for SqlxStorage {
    async fn save_signal(&self, signal: &Signal) -> Result<(), DomainError> {
        let action_str = format!("{:?}", signal.action);
        let status_str = format!("{:?}", signal.status);
        let timeframe_str = format!("{:?}", signal.timeframe);

        let query = r#"
            INSERT INTO signals (
                id, symbol, action, timeframe, entry_price, stop_loss,
                take_profit_1, take_profit_2, take_profit_3, risk_reward_ratio,
                confidence_score, strategy_name, rationale, status, created_at, expires_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            ) ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status
        "#;

        sqlx::query(query)
            .bind(signal.id)
            .bind(signal.symbol.to_compact_string())
            .bind(action_str)
            .bind(timeframe_str)
            .bind(signal.entry_price)
            .bind(signal.stop_loss)
            .bind(signal.take_profit_1)
            .bind(signal.take_profit_2)
            .bind(signal.take_profit_3)
            .bind(signal.risk_reward_ratio)
            .bind(signal.confidence_score)
            .bind(&signal.strategy_name)
            .bind(&signal.rationale)
            .bind(status_str)
            .bind(signal.created_at)
            .bind(signal.expires_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL save_signal error: {}", e)))?;

        Ok(())
    }

    async fn get_signal(&self, id: Uuid) -> Result<Option<Signal>, DomainError> {
        let query = r#"
            SELECT id, symbol, action, timeframe, entry_price, stop_loss,
                   take_profit_1, take_profit_2, take_profit_3, risk_reward_ratio,
                   confidence_score, strategy_name, rationale, status, created_at, expires_at
            FROM signals WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_signal error: {}", e)))?;

        match row {
            Some(r) => {
                let symbol_str: String = r.get("symbol");
                let symbol = Symbol::from_symbol_str(&symbol_str).ok_or_else(|| {
                    DomainError::AdapterError(format!(
                        "Integritas data korup: Symbol '{}' di database tidak valid",
                        symbol_str
                    ))
                })?;
                Ok(Some(Signal {
                    id: r.get("id"),
                    symbol,
                    action: SignalAction::Buy,
                    timeframe: Timeframe::M15,
                    entry_price: r.get("entry_price"),
                    stop_loss: r.get("stop_loss"),
                    take_profit_1: r.get("take_profit_1"),
                    take_profit_2: r.get("take_profit_2"),
                    take_profit_3: r.get("take_profit_3"),
                    risk_reward_ratio: r.get("risk_reward_ratio"),
                    confidence_score: r.get("confidence_score"),
                    strategy_name: r.get("strategy_name"),
                    rationale: r.get("rationale"),
                    status: SignalStatus::Active,
                    created_at: r.get("created_at"),
                    expires_at: r.get("expires_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_active_signals(&self) -> Result<Vec<Signal>, DomainError> {
        let query = r#"
            SELECT id, symbol, action, timeframe, entry_price, stop_loss,
                   take_profit_1, take_profit_2, take_profit_3, risk_reward_ratio,
                   confidence_score, strategy_name, rationale, status, created_at, expires_at
            FROM signals WHERE status IN ('Active', 'Pending') ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DomainError::AdapterError(format!("SQL get_active_signals error: {}", e))
            })?;

        let mut signals = Vec::new();
        for r in rows {
            let symbol_str: String = r.get("symbol");
            let symbol = Symbol::from_symbol_str(&symbol_str).ok_or_else(|| {
                DomainError::AdapterError(format!(
                    "Integritas data korup: Symbol '{}' di database tidak valid",
                    symbol_str
                ))
            })?;
            signals.push(Signal {
                id: r.get("id"),
                symbol,
                action: SignalAction::Buy,
                timeframe: Timeframe::M15,
                entry_price: r.get("entry_price"),
                stop_loss: r.get("stop_loss"),
                take_profit_1: r.get("take_profit_1"),
                take_profit_2: r.get("take_profit_2"),
                take_profit_3: r.get("take_profit_3"),
                risk_reward_ratio: r.get("risk_reward_ratio"),
                confidence_score: r.get("confidence_score"),
                strategy_name: r.get("strategy_name"),
                rationale: r.get("rationale"),
                status: SignalStatus::Active,
                created_at: r.get("created_at"),
                expires_at: r.get("expires_at"),
            });
        }
        Ok(signals)
    }

    async fn save_candles(&self, candles: &[Candle]) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO candles (symbol, timeframe, timestamp, source, open, high, low, close, volume, spread, real_volume)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (symbol, timeframe, timestamp, source) DO UPDATE SET
                close = EXCLUDED.close,
                volume = EXCLUDED.volume,
                spread = COALESCE(EXCLUDED.spread, candles.spread),
                real_volume = COALESCE(EXCLUDED.real_volume, candles.real_volume)
        "#;

        for c in candles {
            let symbol_str = c.symbol.to_compact_string();
            let timeframe_str = format!("{:?}", c.timeframe);
            let source_str = format!("{:?}", c.source);

            sqlx::query(query)
                .bind(symbol_str)
                .bind(timeframe_str)
                .bind(c.timestamp)
                .bind(source_str)
                .bind(c.open)
                .bind(c.high)
                .bind(c.low)
                .bind(c.close)
                .bind(c.volume)
                .bind(c.spread)
                .bind(c.real_volume)
                .execute(&self.pool)
                .await
                .map_err(|e| DomainError::AdapterError(format!("SQL save_candles error: {}", e)))?;
        }
        Ok(())
    }

    async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let symbol_str = symbol.to_compact_string();
        let timeframe_str = format!("{:?}", timeframe);
        let limit_i64 = limit as i64;

        let query = r#"
            SELECT symbol, timeframe, timestamp, source, open, high, low, close, volume, spread, real_volume
            FROM candles
            WHERE symbol = $1 AND timeframe = $2
            ORDER BY timestamp DESC
            LIMIT $3
        "#;

        let rows = sqlx::query(query)
            .bind(symbol_str)
            .bind(timeframe_str)
            .bind(limit_i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_candles error: {}", e)))?;

        let mut candles = Vec::new();
        for r in rows {
            let src_str: String = r
                .try_get("source")
                .unwrap_or_else(|_| "DukascopyEcn".to_string());
            let source = match src_str.as_str() {
                "Mt5BrokerLive" => domain::models::MarketDataSource::Mt5BrokerLive,
                "MrgRealMt4" => domain::models::MarketDataSource::MrgRealMt4,
                "MrgDemoMt4" => domain::models::MarketDataSource::MrgDemoMt4,
                "CtraderOpenApi" => domain::models::MarketDataSource::CtraderOpenApi,
                "SyntheticTest" => domain::models::MarketDataSource::SyntheticTest,
                _ => domain::models::MarketDataSource::DukascopyEcn,
            };

            candles.push(Candle {
                symbol: symbol.clone(),
                timeframe,
                timestamp: r.get("timestamp"),
                source,
                open: r.get("open"),
                high: r.get("high"),
                low: r.get("low"),
                close: r.get("close"),
                volume: r.get("volume"),
                spread: r.try_get("spread").ok(),
                real_volume: r.try_get("real_volume").ok(),
            });
        }
        Ok(candles)
    }

    async fn get_high_watermark(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: domain::models::MarketDataSource,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError> {
        let symbol_str = symbol.to_compact_string();
        let timeframe_str = format!("{:?}", timeframe);
        let source_str = format!("{:?}", source);

        let query = r#"
            SELECT MAX(timestamp) as max_ts
            FROM candles
            WHERE symbol = $1 AND timeframe = $2 AND source = $3
        "#;

        let row = sqlx::query(query)
            .bind(symbol_str)
            .bind(timeframe_str)
            .bind(source_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                DomainError::AdapterError(format!("SQL get_high_watermark error: {}", e))
            })?;

        match row {
            Some(r) => Ok(r.try_get("max_ts").ok()),
            None => Ok(None),
        }
    }

    async fn get_historical_candles_range(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        source: domain::models::MarketDataSource,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let symbol_str = symbol.to_compact_string();
        let timeframe_str = format!("{:?}", timeframe);
        let source_str = format!("{:?}", source);

        let query = r#"
            SELECT symbol, timeframe, timestamp, source, open, high, low, close, volume, spread, real_volume
            FROM candles
            WHERE symbol = $1 AND timeframe = $2 AND source = $3 AND timestamp >= $4 AND timestamp <= $5
            ORDER BY timestamp ASC
        "#;

        let rows = sqlx::query(query)
            .bind(symbol_str)
            .bind(timeframe_str)
            .bind(source_str)
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_historical_candles_range error: {}", e)))?;

        let mut candles = Vec::with_capacity(rows.len());
        for r in rows {
            let symbol_s: String = r.get("symbol");
            let sym = Symbol::from_symbol_str(&symbol_s).ok_or_else(|| {
                DomainError::AdapterError(format!("Symbol corrupt: {}", symbol_s))
            })?;
            let tf_s: String = r.get("timeframe");
            let tf = match tf_s.as_str() {
                "H1" => Timeframe::H1,
                "H4" => Timeframe::H4,
                "M30" => Timeframe::M30,
                "M15" => Timeframe::M15,
                "M5" => Timeframe::M5,
                "M1" => Timeframe::M1,
                "D1" => Timeframe::D1,
                _ => Timeframe::H1,
            };
            let src_s: String = r.get("source");
            let src = domain::models::MarketDataSource::from_source_str(&src_s)
                .unwrap_or(domain::models::MarketDataSource::DukascopyEcn);

            candles.push(Candle {
                symbol: sym,
                timeframe: tf,
                timestamp: r.get("timestamp"),
                source: src,
                open: r.get("open"),
                high: r.get("high"),
                low: r.get("low"),
                close: r.get("close"),
                volume: r.get("volume"),
                spread: r.try_get("spread").ok(),
                real_volume: r.try_get("real_volume").ok(),
            });
        }
        Ok(candles)
    }

    async fn save_order(&self, order: &Order) -> Result<(), DomainError> {
        let action_str = format!("{:?}", order.action);
        let symbol_str = order.symbol.to_compact_string();

        let query = r#"
            INSERT INTO orders (
                id, symbol, action, volume_lots, open_price, current_price,
                stop_loss, take_profit, open_time, close_time, realized_pnl
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            ) ON CONFLICT (id) DO UPDATE SET
                current_price = EXCLUDED.current_price,
                close_time = EXCLUDED.close_time,
                realized_pnl = EXCLUDED.realized_pnl
        "#;

        sqlx::query(query)
            .bind(order.id)
            .bind(symbol_str)
            .bind(action_str)
            .bind(order.volume_lots)
            .bind(order.open_price)
            .bind(order.current_price)
            .bind(order.stop_loss)
            .bind(order.take_profit)
            .bind(order.open_time)
            .bind(order.close_time)
            .bind(order.realized_pnl)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL save_order error: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl TickStoragePort for SqlxStorage {
    async fn save_ticks(&self, ticks: &[Tick]) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO ticks (symbol, timestamp, source, bid, ask, spread_pts, volume)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (symbol, source, timestamp) DO NOTHING
        "#;

        for t in ticks {
            let symbol_str = t.symbol.to_compact_string();
            let source_str = format!("{:?}", t.source);
            let spread_pts_i32 = t.spread_pts.map(|p| p as i32);

            sqlx::query(query)
                .bind(symbol_str)
                .bind(t.timestamp)
                .bind(source_str)
                .bind(t.bid)
                .bind(t.ask)
                .bind(spread_pts_i32)
                .bind(t.volume)
                .execute(&self.pool)
                .await
                .map_err(|e| DomainError::AdapterError(format!("SQL save_ticks error: {}", e)))?;
        }
        Ok(())
    }

    async fn get_ticks(
        &self,
        symbol: &Symbol,
        source: domain::models::MarketDataSource,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Tick>, DomainError> {
        let symbol_str = symbol.to_compact_string();
        let source_str = format!("{:?}", source);

        let query = r#"
            SELECT symbol, timestamp, source, bid, ask, spread_pts, volume
            FROM ticks
            WHERE symbol = $1 AND source = $2 AND timestamp >= $3 AND timestamp <= $4
            ORDER BY timestamp ASC
        "#;

        let rows = sqlx::query(query)
            .bind(symbol_str)
            .bind(source_str)
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_ticks error: {}", e)))?;

        let mut ticks = Vec::new();
        for r in rows {
            let spread_pts: Option<i32> = r.try_get("spread_pts").ok();
            ticks.push(Tick {
                symbol: symbol.clone(),
                timestamp: r.get("timestamp"),
                source,
                bid: r.get("bid"),
                ask: r.get("ask"),
                spread_pts: spread_pts.map(|p| p as u32),
                volume: r.try_get("volume").ok(),
            });
        }
        Ok(ticks)
    }

    async fn get_tick_high_watermark(
        &self,
        symbol: &Symbol,
        source: domain::models::MarketDataSource,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError> {
        let symbol_str = symbol.to_compact_string();
        let source_str = format!("{:?}", source);

        let query = r#"
            SELECT MAX(timestamp) as max_ts
            FROM ticks
            WHERE symbol = $1 AND source = $2
        "#;

        let row = sqlx::query(query)
            .bind(symbol_str)
            .bind(source_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_tick_high_watermark error: {}", e)))?;

        match row {
            Some(r) => Ok(r.try_get("max_ts").ok()),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl BrokerSpecPort for SqlxStorage {
    async fn save_broker_spec(&self, spec: &BrokerSymbolSpec) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO broker_symbol_specs (
                symbol, raw_ticker, digits, point, tick_value, tick_size,
                contract_size, swap_long, swap_short, swap_rollover_3days,
                stop_level_pts, freeze_level_pts, min_lot, lot_step, max_lot, margin_required, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW()
            ) ON CONFLICT (symbol) DO UPDATE SET
                raw_ticker = EXCLUDED.raw_ticker,
                digits = EXCLUDED.digits,
                point = EXCLUDED.point,
                tick_value = EXCLUDED.tick_value,
                tick_size = EXCLUDED.tick_size,
                contract_size = EXCLUDED.contract_size,
                swap_long = EXCLUDED.swap_long,
                swap_short = EXCLUDED.swap_short,
                swap_rollover_3days = EXCLUDED.swap_rollover_3days,
                stop_level_pts = EXCLUDED.stop_level_pts,
                freeze_level_pts = EXCLUDED.freeze_level_pts,
                min_lot = EXCLUDED.min_lot,
                lot_step = EXCLUDED.lot_step,
                max_lot = EXCLUDED.max_lot,
                margin_required = EXCLUDED.margin_required,
                updated_at = NOW()
        "#;

        sqlx::query(query)
            .bind(spec.symbol.to_compact_string())
            .bind(&spec.raw_ticker)
            .bind(spec.digits as i32)
            .bind(spec.point)
            .bind(spec.tick_value)
            .bind(spec.tick_size)
            .bind(spec.contract_size)
            .bind(spec.swap_long)
            .bind(spec.swap_short)
            .bind(spec.swap_rollover_3days as i32)
            .bind(spec.stop_level_pts as i32)
            .bind(spec.freeze_level_pts as i32)
            .bind(spec.min_lot)
            .bind(spec.lot_step)
            .bind(spec.max_lot)
            .bind(spec.margin_required)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL save_broker_spec error: {}", e)))?;

        Ok(())
    }

    async fn get_broker_spec(&self, symbol: &Symbol) -> Result<Option<BrokerSymbolSpec>, DomainError> {
        let query = r#"
            SELECT symbol, raw_ticker, digits, point, tick_value, tick_size,
                   contract_size, swap_long, swap_short, swap_rollover_3days,
                   stop_level_pts, freeze_level_pts, min_lot, lot_step, max_lot, margin_required
            FROM broker_symbol_specs
            WHERE symbol = $1
        "#;

        let row = sqlx::query(query)
            .bind(symbol.to_compact_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL get_broker_spec error: {}", e)))?;

        match row {
            Some(r) => {
                let digits_i32: i32 = r.get("digits");
                let swap_3d_i32: i32 = r.get("swap_rollover_3days");
                let stop_lvl_i32: i32 = r.get("stop_level_pts");
                let freeze_lvl_i32: i32 = r.get("freeze_level_pts");
                Ok(Some(BrokerSymbolSpec {
                    symbol: symbol.clone(),
                    raw_ticker: r.get("raw_ticker"),
                    digits: digits_i32 as u32,
                    point: r.get("point"),
                    tick_value: r.get("tick_value"),
                    tick_size: r.get("tick_size"),
                    contract_size: r.get("contract_size"),
                    swap_long: r.get("swap_long"),
                    swap_short: r.get("swap_short"),
                    swap_rollover_3days: swap_3d_i32 as u32,
                    stop_level_pts: stop_lvl_i32 as u32,
                    freeze_level_pts: freeze_lvl_i32 as u32,
                    min_lot: r.get("min_lot"),
                    lot_step: r.get("lot_step"),
                    max_lot: r.get("max_lot"),
                    margin_required: r.get("margin_required"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn list_broker_specs(&self) -> Result<Vec<BrokerSymbolSpec>, DomainError> {
        let query = r#"
            SELECT symbol, raw_ticker, digits, point, tick_value, tick_size,
                   contract_size, swap_long, swap_short, swap_rollover_3days,
                   stop_level_pts, freeze_level_pts, min_lot, lot_step, max_lot, margin_required
            FROM broker_symbol_specs
            ORDER BY symbol ASC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::AdapterError(format!("SQL list_broker_specs error: {}", e)))?;

        let mut specs = Vec::new();
        for r in rows {
            let symbol_str: String = r.get("symbol");
            if let Some(sym) = Symbol::from_symbol_str(&symbol_str) {
                let digits_i32: i32 = r.get("digits");
                let swap_3d_i32: i32 = r.get("swap_rollover_3days");
                let stop_lvl_i32: i32 = r.get("stop_level_pts");
                let freeze_lvl_i32: i32 = r.get("freeze_level_pts");
                specs.push(BrokerSymbolSpec {
                    symbol: sym,
                    raw_ticker: r.get("raw_ticker"),
                    digits: digits_i32 as u32,
                    point: r.get("point"),
                    tick_value: r.get("tick_value"),
                    tick_size: r.get("tick_size"),
                    contract_size: r.get("contract_size"),
                    swap_long: r.get("swap_long"),
                    swap_short: r.get("swap_short"),
                    swap_rollover_3days: swap_3d_i32 as u32,
                    stop_level_pts: stop_lvl_i32 as u32,
                    freeze_level_pts: freeze_lvl_i32 as u32,
                    min_lot: r.get("min_lot"),
                    lot_step: r.get("lot_step"),
                    max_lot: r.get("max_lot"),
                    margin_required: r.get("margin_required"),
                });
            }
        }
        Ok(specs)
    }
}


// ==============================================================================
// 3. DUCKDB ANALYTICAL ENGINE (In-Process OLAP & Parquet Backtesting Engine)
// ==============================================================================

pub struct DuckDbAnalyticalEngine {
    pub db_path: String,
}

impl DuckDbAnalyticalEngine {
    pub fn new(db_path: impl Into<String>) -> Self {
        Self {
            db_path: db_path.into(),
        }
    }

    /// Query analisis cepat langsung dari file Parquet
    pub fn build_parquet_scan_query(&self, parquet_path: &str, symbol: &str) -> String {
        format!(
            "SELECT timestamp, open, high, low, close, volume FROM read_parquet('{}') WHERE symbol = '{}' ORDER BY timestamp ASC",
            parquet_path, symbol
        )
    }
}
