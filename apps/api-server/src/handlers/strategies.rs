use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct StrategyDto {
    pub id: &'static str,
    pub name: &'static str,
    pub code: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub author: &'static str,
    pub win_rate_pct: f64,
    pub profit_factor: f64,
    pub recovery_factor: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub wfer_pct: f64,
    pub is_tf_compliant: bool,
}

pub async fn strategies_handler() -> Json<Vec<StrategyDto>> {
    let list = vec![
        StrategyDto {
            id: "pola-n-core",
            name: "TF Pola N Structure Engine",
            code: "STRAT_POLA_N_V1",
            description: "Strategi fraktal multi-dekade berbasis konfirmasi swing L1-H1-L2 dan retest Golden Zone 50.0% - 61.8%.",
            category: "MARKET_STRUCTURE",
            author: "TF Quantitative Lab",
            win_rate_pct: 68.4,
            profit_factor: 2.34,
            recovery_factor: 9.80,
            sharpe_ratio: 2.14,
            sortino_ratio: 3.42,
            calmar_ratio: 4.12,
            wfer_pct: 94.8,
            is_tf_compliant: true,
        },
        StrategyDto {
            id: "dual-ema-trend",
            name: "TF Dual EMA Dynamic Trend",
            code: "STRAT_EMA_TREND_V2",
            description: "Sistem pengikut tren dinamis EMA 20 & 50 dengan filter slope momentum dan konfirmasi volume interbank.",
            category: "TREND_FOLLOWING",
            author: "TF Quantitative Lab",
            win_rate_pct: 62.1,
            profit_factor: 2.18,
            recovery_factor: 8.45,
            sharpe_ratio: 1.95,
            sortino_ratio: 2.89,
            calmar_ratio: 3.65,
            wfer_pct: 88.2,
            is_tf_compliant: true,
        },
        StrategyDto {
            id: "liquidity-order-block",
            name: "TF Institutional Order Block & FVG",
            code: "STRAT_ICT_OB_V1",
            description: "Eksploitasi area Fair Value Gap (FVG) dan mitigasi institutional liquidity pool pasca sweep high/low.",
            category: "INSTITUTIONAL_LIQUIDITY",
            author: "TF Quantitative Lab",
            win_rate_pct: 71.0,
            profit_factor: 2.52,
            recovery_factor: 11.20,
            sharpe_ratio: 2.45,
            sortino_ratio: 3.90,
            calmar_ratio: 5.08,
            wfer_pct: 91.5,
            is_tf_compliant: true,
        },
    ];
    Json(list)
}
