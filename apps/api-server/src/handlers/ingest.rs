use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use domain::models::{Symbol, Timeframe};
use domain::ports::ingestion::{
    IngestionRequest, IngestionResult, MarketIngestionPort, SymbolStatusDto,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::state::AppState;

fn default_h1() -> String {
    "H1".to_string()
}

fn default_from_year() -> i32 {
    2015
}

fn default_to_year() -> i32 {
    2026
}

#[derive(serde::Deserialize)]
pub struct IngestPayload {
    pub symbol: String,
    #[serde(default = "default_h1")]
    pub timeframe: String,
    #[serde(default = "default_from_year")]
    pub from_year: i32,
    #[serde(default = "default_to_year")]
    pub to_year: i32,
}

/// Handler `POST /api/market/ingest`
/// Mengunduh data historis secara idempoten dari Dukascopy Bank SA dan me-reload cache memori live
pub async fn market_ingest_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestPayload>,
) -> Result<Json<IngestionResult>, (StatusCode, String)> {
    let sym = Symbol::from_symbol_str(&payload.symbol).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("Format simbol tidak valid: {}", payload.symbol),
        )
    })?;

    let tf = match payload.timeframe.to_uppercase().as_str() {
        "H1" => Timeframe::H1,
        "H4" => Timeframe::H4,
        "D1" => Timeframe::D1,
        "M15" => Timeframe::M15,
        "M5" => Timeframe::M5,
        "M1" => Timeframe::M1,
        _ => Timeframe::H1,
    };

    let req = IngestionRequest {
        symbol: sym.clone(),
        timeframe: tf,
        from_year: payload.from_year,
        to_year: payload.to_year,
    };

    info!(
        "Menerima request ingesti API untuk {} ({}-{})",
        sym, req.from_year, req.to_year
    );

    match tokio::time::timeout(
        Duration::from_secs(120),
        state.ingestion_service.ingest_symbol(req),
    )
    .await
    {
        Ok(Ok(result)) => {
            // Live Hot-Reload memori adapter
            let _ = state.market_adapter.reload_symbol(&sym);
            info!(
                "Ingesti sukses untuk {}: {} bar tersimpan.",
                sym, result.total_candles
            );
            Ok(Json(result))
        }
        Ok(Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Gagal ingesti data: {}", e),
        )),
        Err(_) => Err((
            StatusCode::GATEWAY_TIMEOUT,
            "Proses ingesti melebihi batas waktu (120s)".to_string(),
        )),
    }
}

/// Handler `GET /api/market/symbols`
/// Menampilkan status ketersediaan data seluruh instrumen yang terdaftar
pub async fn market_symbols_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SymbolStatusDto>>, StatusCode> {
    match state.ingestion_service.list_available_symbols().await {
        Ok(symbols) => Ok(Json(symbols)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
