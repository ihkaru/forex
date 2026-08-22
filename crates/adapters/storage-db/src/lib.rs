use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use domain::errors::DomainError;
use domain::models::{Candle, Order, Signal, SignalAction, SignalStatus, Symbol, Timeframe};
use domain::ports::StoragePort;

// ==============================================================================
// 1. IN-MEMORY STORAGE (Fast Prototyping & Unit Testing Double)
// ==============================================================================

#[derive(Default, Clone)]
pub struct InMemoryStorage {
    signals: Arc<RwLock<HashMap<Uuid, Signal>>>,
    candles: Arc<RwLock<Vec<Candle>>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
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
        Ok(map
            .values()
            .filter(|s| s.status == SignalStatus::Active)
            .cloned()
            .collect())
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

    async fn save_order(&self, order: &Order) -> Result<(), DomainError> {
        let mut map = self.orders.write().await;
        map.insert(order.id, order.clone());
        Ok(())
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
            FROM signals WHERE status = 'Active' ORDER BY created_at DESC
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
            INSERT INTO candles (symbol, timeframe, timestamp, source, open, high, low, close, volume)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (symbol, timeframe, timestamp, source) DO UPDATE SET
                close = EXCLUDED.close,
                volume = EXCLUDED.volume
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
            SELECT symbol, timeframe, timestamp, source, open, high, low, close, volume
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
