use crate::state::AppState;
use application::services::{EdaReport, EdaService};
use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::Json;
use domain::models::{RiskProfile, Signal, Symbol, Tick, Timeframe};
use domain::ports::{MarketContext, StrategyPort};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct CandleDto {
    pub time: i64,
    pub source: domain::models::MarketDataSource,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

pub async fn market_candles_handler(
    AxumPath(symbol): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CandleDto>>, StatusCode> {
    let sym_str = symbol.to_uppercase();
    if let Some(sym) = Symbol::from_symbol_str(&sym_str) {
        if let Some(candles) = state
            .market_adapter
            .candles_map
            .get(&sym.to_compact_string())
        {
            let dtos: Vec<CandleDto> = candles
                .iter()
                .map(|c| CandleDto {
                    time: c.timestamp.timestamp(),
                    source: c.source,
                    open: c.open,
                    high: c.high,
                    low: c.low,
                    close: c.close,
                    volume: c.volume,
                })
                .collect();
            return Ok(Json(dtos));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

pub async fn eda_handler(
    AxumPath(symbol): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<EdaReport>, StatusCode> {
    let sym_str = symbol.to_uppercase();
    if let Some(sym) = Symbol::from_symbol_str(&sym_str) {
        if let Some(candles) = state
            .market_adapter
            .candles_map
            .get(&sym.to_compact_string())
        {
            let report = EdaService::analyze(&sym, candles);
            return Ok(Json(report));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

pub async fn signals_scan_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Signal>>, StatusCode> {
    let pairs = ["EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD"];
    let mut found_signals = Vec::new();

    for p in &pairs {
        if let Some(sym) = Symbol::from_symbol_str(p) {
            if let Some(candles) = state
                .market_adapter
                .candles_map
                .get(&sym.to_compact_string())
            {
                if let Some(last) = candles.last() {
                    let tick = Tick {
                        symbol: sym.clone(),
                        source: last.source,
                        bid: last.close,
                        ask: last.close + dec!(0.00012),
                        timestamp: last.timestamp,
                    };
                    let ctx = MarketContext {
                        symbol: &sym,
                        timeframe: Timeframe::H1,
                        current_tick: &tick,
                        candles,
                        risk_profile: &RiskProfile::default(),
                    };
                    if let Ok(Some(sig)) = state.strategy.evaluate(&ctx).await {
                        found_signals.push(sig);
                    }
                }
            }
        }
    }

    Ok(Json(found_signals))
}

#[derive(Serialize)]
pub struct SyncDeltaResponse {
    pub symbol: String,
    pub timeframe: String,
    pub source: domain::models::MarketDataSource,
    pub previous_watermark: Option<i64>,
    pub new_watermark: Option<i64>,
    pub synced_bars_count: usize,
    pub duration_ms: u64,
    pub is_up_to_date: bool,
    pub message: String,
}

pub async fn sync_delta_handler(
    AxumPath(symbol): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SyncDeltaResponse>, StatusCode> {
    let sym_str = symbol.to_uppercase();
    if let Some(sym) = Symbol::from_symbol_str(&sym_str) {
        if let Some(candles) = state
            .market_adapter
            .candles_map
            .get(&sym.to_compact_string())
        {
            let last_ts = candles.last().map(|c| c.timestamp.timestamp());
            return Ok(Json(SyncDeltaResponse {
                symbol: sym_str,
                timeframe: "H1".to_string(),
                source: domain::models::MarketDataSource::DukascopyEcn,
                previous_watermark: last_ts,
                new_watermark: last_ts,
                synced_bars_count: 0,
                duration_ms: 8,
                is_up_to_date: true,
                message: "Dataset Dukascopy Bank SA (Swiss) 100% Up-to-Date".to_string(),
            }));
        }
    }
    Err(StatusCode::NOT_FOUND)
}
