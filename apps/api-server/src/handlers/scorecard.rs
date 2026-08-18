use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use crate::state::AppState;

#[derive(Serialize)]
pub struct PillarScoreDto {
    pub code: &'static str,
    pub name: &'static str,
    pub weight_pct: f64,
    pub score: u8,
    pub max_score: u8,
    pub status: &'static str,
    pub value_label: String,
}

#[derive(Serialize)]
pub struct ScorecardResponse {
    pub total_score: u8,
    pub max_score: u8,
    pub channel_level: &'static str,
    pub partnership_status: &'static str,
    pub revenue_sharing_eligible: bool,
    pub pillars: Vec<PillarScoreDto>,
}

pub async fn scorecard_handler(
    State(_state): State<Arc<AppState>>,
) -> Json<ScorecardResponse> {
    let pillars = vec![
        PillarScoreDto {
            code: "RF",
            name: "Recovery Factor",
            weight_pct: 23.53,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: "RF >= 8.00 (Max Drawdown Containment)".to_string(),
        },
        PillarScoreDto {
            code: "PF",
            name: "Profit Factor",
            weight_pct: 17.65,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: "PF >= 2.10 (6 Bulan Terakhir)".to_string(),
        },
        PillarScoreDto {
            code: "PR",
            name: "Status Kemitraan",
            weight_pct: 17.65,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: "Priority Channel".to_string(),
        },
        PillarScoreDto {
            code: "LG",
            name: "Level Channel",
            weight_pct: 17.65,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: "Legend Analyst Tier".to_string(),
        },
        PillarScoreDto {
            code: "LR",
            name: "Monthly Loss Ratio",
            weight_pct: 11.76,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: "0% - 10% Drawdown Containment".to_string(),
        },
        PillarScoreDto {
            code: "PM",
            name: "Profit Months",
            weight_pct: 5.88,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: "6/6 Bulan Berturut-turut".to_string(),
        },
        PillarScoreDto {
            code: "SB",
            name: "Subscriber Base",
            weight_pct: 5.88,
            score: 4,
            max_score: 4,
            status: "PASSED",
            value_label: ">= 501 Priority VIPs".to_string(),
        },
    ];

    let total: u8 = pillars.iter().map(|p| p.score).sum();

    Json(ScorecardResponse {
        total_score: total,
        max_score: 28,
        channel_level: "LEGEND",
        partnership_status: "PRIORITY",
        revenue_sharing_eligible: true,
        pillars,
    })
}
