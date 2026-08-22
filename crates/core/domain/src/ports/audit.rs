use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::models::{MarketDataSource, Symbol, Timeframe};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarAuditItem {
    pub name: String,
    pub weight_pct: Decimal,
    pub max_points: u32,
    pub achieved_points: u32,
    pub benchmark_rule: String,
    pub our_value: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardAuditReport {
    pub total_score: u32,
    pub max_score: u32,
    pub score_pct: Decimal,
    pub revenue_share_tier: String,
    pub max_revenue_share_pct: u32,
    pub pillars: Vec<PillarAuditItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardAuditReport {
    pub in_sample_bars: usize,
    pub out_of_sample_bars: usize,
    pub total_verified_bars: usize,
    pub in_sample_trades: usize,
    pub out_of_sample_trades: usize,
    pub in_sample_win_rate_pct: Decimal,
    pub out_of_sample_win_rate_pct: Decimal,
    pub in_sample_valued_pips: Decimal,
    pub out_of_sample_valued_pips: Decimal,
    pub wfer_pct: Decimal,
    pub stability_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAuditItem {
    pub id: String,
    pub symbol: String,
    pub action: String,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
    pub open_time: DateTime<Utc>,
    pub open_epoch: i64,
    pub close_time: DateTime<Utc>,
    pub close_epoch: i64,
    pub duration_hours: i64,
    pub pnl_pips: Decimal,
    pub valued_pips: Decimal,
    pub exit_reason: String,
    pub running_equity_pips: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyBreakdownItem {
    pub year: i32,
    pub month: u32,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: Decimal,
    pub gross_profit_pips: Decimal,
    pub gross_loss_pips: Decimal,
    pub net_pips: Decimal,
    pub valued_pips: Decimal,
    pub profit_factor: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPointAudit {
    pub time_epoch: i64,
    pub time_iso: DateTime<Utc>,
    pub equity_pips: Decimal,
    pub equity_vp: Decimal,
    pub drawdown_pips: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloAuditReport {
    pub symbol: String,
    pub iterations: usize,
    pub risk_of_ruin_pct: Decimal,
    pub worst_case_max_dd_vp: Decimal,
    pub median_ending_vp: Decimal,
    pub best_case_p95_vp: Decimal,
    pub confidence_interval_95: (Decimal, Decimal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProvenanceAudit {
    pub symbol: String,
    pub source: MarketDataSource,
    pub provider_name: String,
    pub total_bars: usize,
    pub timeframe: Timeframe,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub high_watermark: Option<DateTime<Utc>>,
    pub mathematical_integrity_pct: Decimal,
    pub zero_mock_verified: bool,
    pub storage_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePairAuditReport {
    pub symbol: Symbol,
    pub tier: u8,
    pub multiplier: Decimal,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: Decimal,
    pub total_raw_pips: Decimal,
    pub total_valued_pips: Decimal,
    pub gross_profit_pips: Decimal,
    pub gross_loss_pips: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown_pips: Decimal,
    pub max_drawdown_vp: Decimal,
    pub recovery_factor: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub is_tf_qualified: bool,
    pub provenance: DataProvenanceAudit,
    pub monte_carlo: MonteCarloAuditReport,
    pub monthly_breakdown: Vec<MonthlyBreakdownItem>,
    pub equity_curve: Vec<EquityPointAudit>,
    pub trades: Vec<TradeAuditItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeActionFilter {
    All,
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeResultFilter {
    All,
    Win,
    Loss,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeExitFilter {
    All,
    TakeProfit,
    StopLoss,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSortField {
    CloseTime,
    OpenTime,
    PnlPips,
    ValuedPips,
    DurationHours,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeFilterQuery {
    pub symbol: Symbol,
    pub action: Option<TradeActionFilter>,
    pub result: Option<TradeResultFilter>,
    pub exit_reason: Option<TradeExitFilter>,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub min_pnl_pips: Option<Decimal>,
    pub max_pnl_pips: Option<Decimal>,
    pub min_valued_pips: Option<Decimal>,
    pub min_duration_hours: Option<i64>,
    pub max_duration_hours: Option<i64>,
    pub sort_by: Option<TradeSortField>,
    pub sort_direction: Option<SortDirection>,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredTradesSummary {
    pub matched_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: Decimal,
    pub total_raw_pips: Decimal,
    pub total_valued_pips: Decimal,
    pub gross_profit_pips: Decimal,
    pub gross_loss_pips: Decimal,
    pub profit_factor: Decimal,
    pub avg_trade_pips: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedTradesResponse {
    pub symbol: Symbol,
    pub total_records: usize,
    pub total_pages: usize,
    pub current_page: usize,
    pub page_size: usize,
    pub has_next_page: bool,
    pub has_prev_page: bool,
    pub summary: FilteredTradesSummary,
    pub trades: Vec<TradeAuditItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullQuantAuditReport {
    pub generated_at: DateTime<Utc>,
    pub total_portfolio_valued_pips: Decimal,
    pub monthly_tf_target_vp: Decimal,
    pub is_portfolio_tf_qualified: bool,
    pub portfolio_win_rate_pct: Decimal,
    pub total_portfolio_trades: usize,
    pub scorecard: ScorecardAuditReport,
    pub walk_forward: WalkForwardAuditReport,
    pub pairs: Vec<SinglePairAuditReport>,
}

#[async_trait]
pub trait QuantAuditPort: Send + Sync {
    async fn get_full_audit(&self) -> Result<FullQuantAuditReport, DomainError>;
    async fn get_pair_audit(&self, symbol: &Symbol) -> Result<SinglePairAuditReport, DomainError>;
    async fn get_paginated_trades(
        &self,
        query: &TradeFilterQuery,
    ) -> Result<PaginatedTradesResponse, DomainError>;
}
