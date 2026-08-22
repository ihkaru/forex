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
    pub action: Option<String>,
    pub result: Option<String>,
    pub exit_reason: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub min_pnl: Option<Decimal>,
    pub max_pnl: Option<Decimal>,
    pub min_vp: Option<Decimal>,
    pub min_duration_hours: Option<i64>,
    pub max_duration_hours: Option<i64>,
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
            .and_then(|a| match a.to_uppercase().as_str() {
                "BUY" | "BUY_LIMIT" | "LONG" => Some(TradeActionFilter::Buy),
                "SELL" | "SELL_LIMIT" | "SHORT" => Some(TradeActionFilter::Sell),
                _ => Some(TradeActionFilter::All),
            });

        let result = params
            .result
            .as_deref()
            .and_then(|r| match r.to_uppercase().as_str() {
                "WIN" | "PROFIT" => Some(TradeResultFilter::Win),
                "LOSS" | "LOSE" => Some(TradeResultFilter::Loss),
                _ => Some(TradeResultFilter::All),
            });

        let exit_reason =
            params
                .exit_reason
                .as_deref()
                .and_then(|e| match e.to_uppercase().as_str() {
                    "TP" | "TAKE_PROFIT" => Some(TradeExitFilter::TakeProfit),
                    "SL" | "STOP_LOSS" => Some(TradeExitFilter::StopLoss),
                    "EXPIRED" | "EXPIRY" => Some(TradeExitFilter::Expired),
                    _ => Some(TradeExitFilter::All),
                });

        let sort_by = params
            .sort_by
            .as_deref()
            .and_then(|s| match s.to_lowercase().as_str() {
                "open" | "open_time" => Some(TradeSortField::OpenTime),
                "pnl" | "pnl_pips" => Some(TradeSortField::PnlPips),
                "vp" | "valued_pips" => Some(TradeSortField::ValuedPips),
                "duration" | "duration_hours" => Some(TradeSortField::DurationHours),
                _ => Some(TradeSortField::CloseTime),
            });

        let sort_direction =
            params
                .sort_direction
                .as_deref()
                .and_then(|d| match d.to_lowercase().as_str() {
                    "asc" | "ascending" => Some(SortDirection::Asc),
                    _ => Some(SortDirection::Desc),
                });

        let query = TradeFilterQuery {
            symbol: sym,
            action,
            result,
            exit_reason,
            year: params.year,
            month: params.month,
            min_pnl_pips: params.min_pnl,
            max_pnl_pips: params.max_pnl,
            min_valued_pips: params.min_vp,
            min_duration_hours: params.min_duration_hours,
            max_duration_hours: params.max_duration_hours,
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
