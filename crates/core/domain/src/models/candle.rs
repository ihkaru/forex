use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MarketDataSource {
    #[default]
    DukascopyEcn,
    Mt5BrokerLive,
    MrgMetaTrader4,
    CtraderOpenApi,
    SyntheticTest,
}

impl MarketDataSource {
    /// Parse string identifier (case-insensitive) ke MarketDataSource enum
    pub fn from_source_str(s: &str) -> Result<Self, crate::errors::DomainError> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "dukascopy" | "dukascopy_ecn" | "dukascopyecn" | "ecn" => Ok(Self::DukascopyEcn),
            "mrg" | "mrg_mt4" | "mrgmetatrader4" | "mt4" => Ok(Self::MrgMetaTrader4),
            "mt5" | "mt5_live" | "mt5brokerlive" | "live" => Ok(Self::Mt5BrokerLive),
            "ctrader" | "ctrader_openapi" | "ctraderopenapi" => Ok(Self::CtraderOpenApi),
            "synthetic" | "synthetic_test" | "synthetictest" | "test" => Ok(Self::SyntheticTest),
            _ => Err(crate::errors::DomainError::ValidationError(format!(
                "Sumber data pasar '{}' tidak valid. Sumber yang didukung: 'dukascopy', 'mrg_mt4', 'mt5_live', 'ctrader'",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DukascopyEcn => "dukascopy",
            Self::MrgMetaTrader4 => "mrg_mt4",
            Self::Mt5BrokerLive => "mt5_live",
            Self::CtraderOpenApi => "ctrader",
            Self::SyntheticTest => "synthetic",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DukascopyEcn => "Dukascopy ECN (10-Yr Historical Tick)",
            Self::MrgMetaTrader4 => "MRG MetaTrader 4 (Broker Live Feed)",
            Self::Mt5BrokerLive => "MetaTrader 5 Live Feed",
            Self::CtraderOpenApi => "cTrader Open API",
            Self::SyntheticTest => "Synthetic Test Data",
        }
    }

    pub fn is_live(&self) -> bool {
        matches!(
            self,
            Self::MrgMetaTrader4 | Self::Mt5BrokerLive | Self::CtraderOpenApi
        )
    }
}

/// Permintaan Lilin Pasar Berstandar Interface-First (TradingView UDF Compliant)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandleQuery {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub source: MarketDataSource, // 🔴 WAJIB NON-OPTIONAL (Data Provenance)
    pub limit: Option<usize>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl CandleQuery {
    pub fn new(symbol: Symbol, timeframe: Timeframe, source: MarketDataSource) -> Self {
        Self {
            symbol,
            timeframe,
            source,
            limit: None,
            from: None,
            to: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_range(mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self.to = Some(to);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub source: MarketDataSource,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: Symbol,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub source: MarketDataSource,
    pub bid: Decimal,
    pub ask: Decimal,
}

impl Candle {
    #[inline]
    pub fn epoch_seconds(&self) -> i64 {
        self.timestamp.timestamp()
    }

    #[inline]
    pub fn epoch_millis(&self) -> i64 {
        self.timestamp.timestamp_millis()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_epoch_seconds(
        symbol: Symbol,
        timeframe: Timeframe,
        epoch_secs: i64,
        source: MarketDataSource,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
    ) -> Option<Self> {
        let timestamp = DateTime::from_timestamp(epoch_secs, 0)?;
        Some(Self {
            symbol,
            timeframe,
            timestamp,
            source,
            open,
            high,
            low,
            close,
            volume,
        })
    }
}

impl Tick {
    #[inline]
    pub fn epoch_seconds(&self) -> i64 {
        self.timestamp.timestamp()
    }

    #[inline]
    pub fn epoch_millis(&self) -> i64 {
        self.timestamp.timestamp_millis()
    }

    pub fn spread(&self) -> Decimal {
        self.ask - self.bid
    }

    pub fn mid_price(&self) -> Decimal {
        (self.bid + self.ask) / rust_decimal_macros::dec!(2)
    }
}
