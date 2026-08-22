use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use application::services::QuantAuditService;
use domain::models::Symbol;
use domain::ports::audit::{FullQuantAuditReport, SinglePairAuditReport};
use domain::ports::QuantAuditPort;

use crate::state::AppState;

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
