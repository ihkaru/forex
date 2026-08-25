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
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let gold = Symbol::new("XAU", "USD");

    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    println!("\n=========================================================================================");
    println!("  POLA N V7 VALKYRIE FINAL PRODUCTION BENCHMARK (10 YEARS XAUUSD & MULTI-PAIR)");
    println!(
        "========================================================================================="
    );

    let strat_v7 = Arc::new(PolaNStrategy::v7_valkyrie_pro());
    let s = BacktestService::new(adapter.clone(), strat_v7, RiskProfile::default());

    if let Ok(rep) = s.run_simulation(&gold, Timeframe::H1, from, to).await {
        println!("👑 POLA N V7 VALKYRIE PRO (XAUUSD 10-TAHUN DUKASCOPY ECN):");
        println!("─────────────────────────────────────────────────────────────────────────────");
        println!("• Total Closed Trades     : {} Trades", rep.total_trades);
        println!("• Win Rate                : {:.1}%", rep.win_rate_percent);
        println!(
            "• Profit Factor           : {:.2} (Target: > 1.65)",
            rep.profit_factor
        );
        println!("• Net Realized Pips       : {:.1} Pips", rep.total_raw_pips);
        println!(
            "• Total Valued Pips (VP)  : {:.1} VP",
            rep.total_valued_pips
        );
        println!(
            "• Max Drawdown            : {:.1} pips",
            rep.max_drawdown_pips
        );
        println!("• Recovery Factor         : {:.2}", rep.recovery_factor);
        println!("─────────────────────────────────────────────────────────────────────────────\n");
    }

    // Walk-Forward Out-of-Sample (2022-2026)
    let oos_from = Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).unwrap();
    if let Ok(rep) = s.run_simulation(&gold, Timeframe::H1, oos_from, to).await {
        println!("🛡️ BLIND OUT-OF-SAMPLE TEST (2022-2026 UNSEEN DATA):");
        println!("─────────────────────────────────────────────────────────────────────────────");
        println!("• Trades (4.5 Thn OOS)    : {} Trades", rep.total_trades);
        println!("• Win Rate OOS            : {:.1}%", rep.win_rate_percent);
        println!("• Profit Factor OOS       : {:.2}", rep.profit_factor);
        println!(
            "• Valued Pips OOS         : {:.1} VP",
            rep.total_valued_pips
        );
        println!("• Recovery Factor OOS     : {:.2}", rep.recovery_factor);
        println!("─────────────────────────────────────────────────────────────────────────────\n");
    }
}
