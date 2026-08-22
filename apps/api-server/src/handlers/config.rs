use axum::Json;
use domain::models::{Symbol, TfPairSpec};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Serialize)]
pub struct PairConfigDto {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub tier: u8,
    pub multiplier: Decimal,
    pub pip_size: Decimal,
    pub min_pips: Decimal,
    pub max_pips: Decimal,
}

#[derive(Serialize)]
pub struct SystemConfigResponse {
    pub active_pairs: Vec<PairConfigDto>,
    pub tf_monthly_goal_vp: Decimal,
    pub tf_point_cash_value_idr: u64,
    pub min_settled_trades_monthly: usize,
    pub max_rr_ratio: Decimal,
}

pub async fn config_handler() -> Json<SystemConfigResponse> {
    let pairs = [
        "EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD", "XAUUSD",
    ];
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
                multiplier: spec.value_multiplier,
                pip_size: spec.pip_size,
                min_pips: spec.min_sl_tp_pips,
                max_pips: spec.max_sl_tp_pips,
            });
        }
    }

    Json(SystemConfigResponse {
        active_pairs,
        tf_monthly_goal_vp: Decimal::from(300),
        tf_point_cash_value_idr: 10_000,
        min_settled_trades_monthly: 20,
        max_rr_ratio: Decimal::from(3),
    })
}
