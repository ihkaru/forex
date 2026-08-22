use axum::extract::{Path as AxumPath, Query, State};
use axum::http::StatusCode;
use axum::Json;
use rust_decimal::Decimal;
use std::sync::Arc;

use application::services::QuantAuditService;
use domain::models::Symbol;
use domain::ports::audit::{
    FullQuantAuditReport, PaginatedTradesResponse, SinglePairAuditReport, SortDirection,
    TradeActionFilter, TradeExitFilter, TradeFilterQuery, TradeResultFilter, TradeSortField,
};
use domain::ports::QuantAuditPort;

use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct TradeQueryParam {
    pub q: Option<String>,
    pub search: Option<String>,
    pub action: Option<String>,
    pub result: Option<String>,
    pub exit_reason: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub min_pnl: Option<Decimal>,
    pub max_pnl: Option<Decimal>,
    pub pnl_gt: Option<Decimal>,
    pub pnl_gte: Option<Decimal>,
    pub pnl_lt: Option<Decimal>,
    pub pnl_lte: Option<Decimal>,
    pub min_vp: Option<Decimal>,
    pub vp_gt: Option<Decimal>,
    pub vp_gte: Option<Decimal>,
    pub vp_lt: Option<Decimal>,
    pub vp_lte: Option<Decimal>,
    pub price_gt: Option<Decimal>,
    pub price_lt: Option<Decimal>,
    pub min_duration_hours: Option<i64>,
    pub max_duration_hours: Option<i64>,
    pub duration_gt: Option<i64>,
    pub duration_lt: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

pub async fn audit_full_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<FullQuantAuditReport>, StatusCode> {
    let audit_service = QuantAuditService::new(
        state.market_adapter.clone(),
        state.strategy.clone(),
        state.storage.clone(),
    );

    match audit_service.get_full_audit().await {
        Ok(report) => Ok(Json(report)),
        Err(e) => {
            eprintln!("❌ Audit full error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn audit_pair_handler(
    AxumPath(symbol): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SinglePairAuditReport>, StatusCode> {
    let sym_str = symbol.to_uppercase();
    if let Some(sym) = Symbol::from_symbol_str(&sym_str) {
        let audit_service = QuantAuditService::new(
            state.market_adapter.clone(),
            state.strategy.clone(),
            state.storage.clone(),
        );

        match audit_service.get_pair_audit(&sym).await {
            Ok(pair_report) => Ok(Json(pair_report)),
            Err(e) => {
                eprintln!("❌ Audit pair {} error: {}", sym_str, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn audit_trades_paginated_handler(
    AxumPath(symbol): AxumPath<String>,
    Query(params): Query<TradeQueryParam>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedTradesResponse>, StatusCode> {
    let sym_str = symbol.to_uppercase();
    if let Some(sym) = Symbol::from_symbol_str(&sym_str) {
        let audit_service = QuantAuditService::new(
            state.market_adapter.clone(),
            state.strategy.clone(),
            state.storage.clone(),
        );

        let action = params
            .action
            .as_deref()
            .map(|a| match a.to_uppercase().as_str() {
                "BUY" | "BUY_LIMIT" | "LONG" => TradeActionFilter::Buy,
                "SELL" | "SELL_LIMIT" | "SHORT" => TradeActionFilter::Sell,
                _ => TradeActionFilter::All,
            });

        let result = params
            .result
            .as_deref()
            .map(|r| match r.to_uppercase().as_str() {
                "WIN" | "PROFIT" => TradeResultFilter::Win,
                "LOSS" | "LOSE" => TradeResultFilter::Loss,
                _ => TradeResultFilter::All,
            });

        let exit_reason = params
            .exit_reason
            .as_deref()
            .map(|e| match e.to_uppercase().as_str() {
                "TP" | "TAKE_PROFIT" => TradeExitFilter::TakeProfit,
                "SL" | "STOP_LOSS" => TradeExitFilter::StopLoss,
                "EXPIRED" | "EXPIRY" => TradeExitFilter::Expired,
                _ => TradeExitFilter::All,
            });

        let search_query = params.search.or(params.q).filter(|s| !s.trim().is_empty());

        let sort_by = params
            .sort_by
            .as_deref()
            .map(|s| match s.to_lowercase().as_str() {
                "#" | "idx" | "index" => TradeSortField::Index,
                "open" | "open_time" | "date_in" => TradeSortField::OpenTime,
                "close" | "close_time" | "date_out" => TradeSortField::CloseTime,
                "action" | "type" | "dir" | "direction" => TradeSortField::Action,
                "open_price" | "price_in" | "entry" => TradeSortField::OpenPrice,
                "close_price" | "price_out" | "exit" => TradeSortField::ClosePrice,
                "pnl" | "pnl_pips" | "net" => TradeSortField::PnlPips,
                "vp" | "valued_pips" => TradeSortField::ValuedPips,
                "duration" | "duration_hours" | "hours" => TradeSortField::DurationHours,
                "status" | "reason" | "exit_reason" => TradeSortField::ExitReason,
                _ => TradeSortField::CloseTime,
            });

        let sort_direction =
            params
                .sort_direction
                .as_deref()
                .map(|d| match d.to_lowercase().as_str() {
                    "asc" | "ascending" => SortDirection::Asc,
                    _ => SortDirection::Desc,
                });

        let query = TradeFilterQuery {
            symbol: sym,
            search_query,
            action,
            result,
            exit_reason,
            year: params.year,
            month: params.month,
            min_pnl_pips: params.min_pnl,
            max_pnl_pips: params.max_pnl,
            pnl_gt: params.pnl_gt,
            pnl_gte: params.pnl_gte,
            pnl_lt: params.pnl_lt,
            pnl_lte: params.pnl_lte,
            min_valued_pips: params.min_vp,
            vp_gt: params.vp_gt,
            vp_gte: params.vp_gte,
            vp_lt: params.vp_lt,
            vp_lte: params.vp_lte,
            price_gt: params.price_gt,
            price_lt: params.price_lt,
            min_duration_hours: params.min_duration_hours,
            max_duration_hours: params.max_duration_hours,
            duration_gt: params.duration_gt,
            duration_lt: params.duration_lt,
            sort_by,
            sort_direction,
            page: params.page.unwrap_or(1),
            page_size: params.limit.unwrap_or(50),
        };

        match audit_service.get_paginated_trades(&query).await {
            Ok(paginated) => Ok(Json(paginated)),
            Err(e) => {
                eprintln!("❌ Audit paginated trades error: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
