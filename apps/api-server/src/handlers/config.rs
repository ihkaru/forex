use axum::Json;
use domain::models::{Symbol, TfPairSpec};
use serde::Serialize;

#[derive(Serialize)]
pub struct PairConfigDto {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub tier: u8,
    pub multiplier: f64,
    pub pip_size: f64,
    pub min_pips: f64,
    pub max_pips: f64,
}

#[derive(Serialize)]
pub struct SystemConfigResponse {
    pub active_pairs: Vec<PairConfigDto>,
    pub tf_monthly_goal_vp: f64,
    pub tf_point_cash_value_idr: u64,
    pub min_settled_trades_monthly: usize,
    pub max_rr_ratio: f64,
}

pub async fn config_handler() -> Json<SystemConfigResponse> {
    let pairs = ["EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD"];
    let mut active_pairs = Vec::new();

    for p in &pairs {
        if let Some(sym) = Symbol::from_symbol_str(p) {
            let spec = TfPairSpec::from_symbol(&sym);
            let tier_num = match spec.tier {
                domain::models::PairTier::Tier1 => 1,
                domain::models::PairTier::Tier2 => 2,
                domain::models::PairTier::Tier3 => 3,
                domain::models::PairTier::Tier4 => 4,
            };
            active_pairs.push(PairConfigDto {
                symbol: p.to_string(),
                base: sym.base.clone(),
                quote: sym.quote.clone(),
                tier: tier_num,
                multiplier: spec.value_multiplier.to_string().parse().unwrap_or(1.0),
                pip_size: spec.pip_size.to_string().parse().unwrap_or(0.0001),
                min_pips: spec.min_sl_tp_pips.to_string().parse().unwrap_or(10.0),
                max_pips: spec.max_sl_tp_pips.to_string().parse().unwrap_or(200.0),
            });
        }
    }

    Json(SystemConfigResponse {
        active_pairs,
        tf_monthly_goal_vp: 300.0,
        tf_point_cash_value_idr: 10_000,
        min_settled_trades_monthly: 5,
        max_rr_ratio: 3.0,
    })
}
