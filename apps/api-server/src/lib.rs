pub mod handlers;
pub mod state;

use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use handlers::*;
use state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/config", get(config_handler))
        .route("/api/scorecard", get(scorecard_handler))
        .route("/api/strategies", get(strategies_handler))
        .route("/api/monte-carlo/:symbol", get(monte_carlo_handler))
        .route("/api/backtest", get(backtest_handler))
        .route("/api/backtest/trades/:symbol", get(backtest_trades_handler))
        .route(
            "/api/backtest/detailed/:symbol",
            get(backtest_detailed_handler),
        )
        .route("/api/eda/:symbol", get(eda_handler))
        .route("/api/market/candles/:symbol", get(market_candles_handler))
        .route("/api/market/sync/:symbol", get(sync_delta_handler))
        .route("/api/signals/scan", get(signals_scan_handler))
        .route("/api/audit/full", get(audit_full_handler))
        .route("/api/audit/pair/:symbol", get(audit_pair_handler))
        .route(
            "/api/audit/trades/:symbol",
            get(audit_trades_paginated_handler),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}
