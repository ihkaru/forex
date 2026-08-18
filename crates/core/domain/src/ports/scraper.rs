use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::models::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Holiday,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicEvent {
    pub title: String,
    pub country_currency: String,
    pub impact: ImpactLevel,
    pub event_time: DateTime<Utc>,
    pub forecast: Option<String>,
    pub previous: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SentimentData {
    pub symbol: Symbol,
    pub long_percentage: Decimal,
    pub short_percentage: Decimal,
    pub total_positions: u64,
    pub fetched_at: DateTime<Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EconomicCalendarPort: Send + Sync {
    async fn fetch_events(&self, date: DateTime<Utc>) -> Result<Vec<EconomicEvent>, DomainError>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SentimentPort: Send + Sync {
    async fn fetch_sentiment(&self, symbol: &Symbol) -> Result<SentimentData, DomainError>;
}
