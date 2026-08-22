use crate::state::AppState;
use application::services::audit::QuantAuditService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use domain::ports::audit::QuantAuditPort;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct PillarScoreDto {
    pub code: String,
    pub name: String,
    pub weight_pct: f64,
    pub score: u32,
    pub max_score: u32,
    pub status: String,
    pub value_label: String,
}

#[derive(Serialize)]
pub struct ScorecardResponse {
    pub total_score: u32,
    pub max_score: u32,
    pub channel_level: String,
    pub partnership_status: &'static str,
    pub revenue_sharing_eligible: bool,
    pub pillars: Vec<PillarScoreDto>,
}

pub async fn scorecard_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ScorecardResponse>, StatusCode> {
    let audit_service = QuantAuditService::new(
        state.market_adapter.clone(),
        state.strategy.clone(),
        state.storage.clone(),
    );

    let full_audit = audit_service
        .get_full_audit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sc = full_audit.scorecard;

    let pillars: Vec<PillarScoreDto> = sc
        .pillars
        .into_iter()
        .map(|p| {
            let weight: f64 = p.weight_pct.to_string().parse().unwrap_or(0.0);
            PillarScoreDto {
                code: p.code,
                name: p.name,
                weight_pct: weight,
                score: p.achieved_points,
                max_score: p.max_points,
                status: p.status,
                value_label: format!("{} (Rule: {})", p.our_value, p.benchmark_rule),
            }
        })
        .collect();

    Ok(Json(ScorecardResponse {
        total_score: sc.total_score,
        max_score: sc.max_score,
        channel_level: sc.revenue_share_tier,
        partnership_status: "PRIORITY_VERIFIED",
        revenue_sharing_eligible: sc.total_score >= 12,
        pillars,
    }))
}
