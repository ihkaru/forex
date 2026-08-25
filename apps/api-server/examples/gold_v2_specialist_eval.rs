#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    clippy::all
)]
use api_server::state::RealHistoricalMarketAdapter;
use application::services::BacktestService;
use chrono::{TimeZone, Utc};
use domain::models::{PolaNStrategy, RiskProfile, Symbol, Timeframe};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let gold = Symbol::new("XAU", "USD");
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    println!("\n🔍 GOLD (XAUUSD) QUANTITATIVE PARAMETER GRID SEARCH (2015 - 2026)");
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "{:<15} | {:<7} | {:<8} | {:<12} | {:<7} | {:<8} | {:<8}",
        "Config", "Trades", "Trd/Bln", "Valued Pips", "WinRate", "PF", "RecFactor"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    let swing_configs = [(5, 3), (4, 3), (4, 2), (6, 3)];
    let rr_configs = [dec!(1.20), dec!(1.30), dec!(1.40), dec!(1.50), dec!(1.60)];

    for (left, right) in swing_configs {
        for rr in rr_configs {
            let label = format!("Sw({left},{right}) RR:{rr}");
            let strategy = Arc::new(PolaNStrategy::with_params(
                &label,
                left,
                right,
                dec!(0.00020),
                rr,
            ));
            let service = BacktestService::new(adapter.clone(), strategy, RiskProfile::default());

            if let Ok(rep) = service.run_simulation(&gold, Timeframe::H1, from, to).await {
                let trd_per_mo = (Decimal::from(rep.total_trades) / dec!(137.0)).round_dp(1);
                println!(
                    "{:<15} | {:<7} | {:<8} | {:>10.1} VP | {:>6.1}% | {:>8.2} | {:>8.2}",
                    label,
                    rep.total_trades,
                    trd_per_mo,
                    rep.total_valued_pips,
                    rep.win_rate_percent,
                    rep.profit_factor,
                    rep.recovery_factor
                );
            }
        }
    }
}
