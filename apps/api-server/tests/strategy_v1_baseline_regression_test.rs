use api_server::state::RealHistoricalMarketAdapter;
use application::services::BacktestService;
use chrono::{TimeZone, Utc};
use domain::models::{PolaNStrategy, RiskProfile, Symbol, Timeframe};
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::test]
async fn test_v1_production_strategy_baseline_invariants() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let v1_strategy = Arc::new(PolaNStrategy::v1_production());
    let service = BacktestService::new(adapter.clone(), v1_strategy, RiskProfile::default());

    let gold = Symbol::new("XAU", "USD");
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let report = service
        .run_simulation(&gold, Timeframe::H1, from, to)
        .await
        .expect("Backtest V1 for XAUUSD must succeed");

    // Invariant 1: Valued Pips harus positif dan profitabel (> 600 VP pada Gold tanpa BE)
    assert!(
        report.total_valued_pips >= dec!(600.0),
        "V1 Production Valued Pips must be >= 600 VP, got {}",
        report.total_valued_pips
    );

    // Invariant 2: Profit Factor harus > 1.15 (Profitable net edge)
    assert!(
        report.profit_factor >= dec!(1.15),
        "V1 Production Profit Factor must be >= 1.15, got {}",
        report.profit_factor
    );

    // Invariant 3: Recovery Factor harus >= 1.0 (Pure TF No-Intervention Structural SL)
    assert!(
        report.recovery_factor >= dec!(1.0),
        "V1 Production Recovery Factor must be >= 1.0, got {}",
        report.recovery_factor
    );
}
