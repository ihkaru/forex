use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::signal::SignalAction;
use super::symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub symbol: Symbol,
    pub action: SignalAction,
    pub volume_lots: Decimal,
    pub open_price: Decimal,
    pub current_price: Decimal,
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
    pub open_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub realized_pnl: Option<Decimal>,
}
