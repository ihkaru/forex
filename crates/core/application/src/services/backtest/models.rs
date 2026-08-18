use chrono::{DateTime, Utc};
use domain::models::{Order, Symbol, Timeframe};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EquityPoint {
    pub time: i64,
    pub equity_pips: Decimal,
    pub drawdown_pips: Decimal,
    pub drawdown_percent: Decimal,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct TradeDirectionBreakdown {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: Decimal,
    pub gross_profit_pips: Decimal,
    pub gross_loss_pips: Decimal,
    pub net_pips: Decimal,
    pub profit_factor: Decimal,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct TradingViewPerformanceSummary {
    pub all: TradeDirectionBreakdown,
    pub long: TradeDirectionBreakdown,
    pub short: TradeDirectionBreakdown,
    pub largest_win_pips: Decimal,
    pub largest_loss_pips: Decimal,
    pub max_consecutive_wins: usize,
    pub max_consecutive_losses: usize,
    pub avg_trade_pips: Decimal,
    pub avg_win_pips: Decimal,
    pub avg_loss_pips: Decimal,
    pub payoff_ratio: Decimal,
    pub avg_bars_held: Decimal,
    pub max_drawdown_pips: Decimal,
    pub max_drawdown_pct: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
}

/// Hasil Laporan Kuantitatif Komprehensif Backtest (Standar Traders Family 7-Faktor)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BacktestReport {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_percent: Decimal,
    pub total_raw_pips: Decimal,
    pub total_valued_pips: Decimal,
    pub gross_profit_pips: Decimal,
    pub gross_loss_pips: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown_pips: Decimal,
    pub recovery_factor: Decimal,
    pub monthly_loss_ratio_percent: Decimal,
    pub is_tf_qualified: bool,
    #[serde(default)]
    pub summary: Option<TradingViewPerformanceSummary>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DetailedBacktestResult {
    pub report: BacktestReport,
    pub trades: Vec<Order>,
    #[serde(default)]
    pub equity_curve: Vec<EquityPoint>,
}

#[derive(Debug, Clone)]
pub(crate) enum SimulatedOrderStatus {
    Pending,
    Filled { fill_time: DateTime<Utc> },
}

#[derive(Debug, Clone)]
pub(crate) struct SimulatedOrder {
    pub(crate) order: Order,
    pub(crate) status: SimulatedOrderStatus,
    pub(crate) expires_at: DateTime<Utc>,
}
