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

    // 70% In-Sample (Train): 2015-01-01 s/d 2021-12-31 (7 Tahun)
    let is_from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let is_to = Utc.with_ymd_and_hms(2021, 12, 31, 23, 59, 59).unwrap();

    // 30% Out-of-Sample (Blind Test): 2022-01-01 s/d 2026-08-01 (4.5 Tahun)
    let oos_from = Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).unwrap();
    let oos_to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    println!("\n=========================================================================================");
    println!(
        "  SCIENTIFIC QUANT RIGOR AUDIT 2026: IN-SAMPLE VS OUT-OF-SAMPLE WALK-FORWARD VALIDATION"
    );
    println!(
        "========================================================================================="
    );
    println!("In-Sample (Train 70%)     : 2015-01-01 s/d 2021-12-31 (7 Tahun)");
    println!(
        "Out-of-Sample (Blind 30%) : 2022-01-01 s/d 2026-08-01 (4.5 Tahun Murni Tanpa Tuning)"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    let strats: Vec<(&str, Arc<PolaNStrategy>)> = vec![
        (
            "V1 Baseline Production",
            Arc::new(PolaNStrategy::v1_production()),
        ),
        (
            "V3 Institutional Baseline",
            Arc::new(PolaNStrategy::v3_gold_pro()),
        ),
        ("V4 Quantum Pro", Arc::new(PolaNStrategy::v4_quantum_pro())),
        (
            "V5 Apex Institutional",
            Arc::new(PolaNStrategy::v5_apex_pro()),
        ),
        (
            "V6 Hyperion Apex",
            Arc::new(PolaNStrategy::v6_hyperion_pro()),
        ),
    ];

    println!(
        "{:<26} | {:<22} | {:<22} | {:<8} | {:<8}",
        "Model Versi", "In-Sample (2015-21)", "Out-of-Sample (2022-26)", "WFER %", "Status"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    for (name, strat) in strats {
        let s = BacktestService::new(adapter.clone(), strat.clone(), RiskProfile::default());

        let is_res = s.run_simulation(&gold, Timeframe::H1, is_from, is_to).await;
        let oos_res = s
            .run_simulation(&gold, Timeframe::H1, oos_from, oos_to)
            .await;

        if let (Ok(is_rep), Ok(oos_rep)) = (is_res, oos_res) {
            let is_annualized_vp = is_rep.total_valued_pips / dec!(7.0);
            let oos_annualized_vp = oos_rep.total_valued_pips / dec!(4.5);

            let wfer = if is_annualized_vp > Decimal::ZERO {
                (oos_annualized_vp / is_annualized_vp) * dec!(100.0)
            } else {
                Decimal::ZERO
            };

            let status = if wfer >= dec!(70.0) && oos_rep.profit_factor >= dec!(1.35) {
                "✅ ROBUST EDGE"
            } else if wfer >= dec!(50.0) {
                "⚠️ MODERATE"
            } else {
                "❌ OVERFITTED"
            };

            println!(
                "{:<26} | {:>7.1} VP (PF {:>4.2}) | {:>7.1} VP (PF {:>4.2}) | {:>6.1}% | {}",
                name,
                is_rep.total_valued_pips,
                is_rep.profit_factor,
                oos_rep.total_valued_pips,
                oos_rep.profit_factor,
                wfer,
                status
            );
        }
    }

    println!("\n=========================================================================================");
    println!("  MULTI-ASSET CROSS-VALIDATION MATRIX (TESTING ROBUSTNESS ACROSS 6 OTHER PAIRS)");
    println!(
        "========================================================================================="
    );

    let test_pairs = vec![
        Symbol::new("EUR", "USD"),
        Symbol::new("GBP", "USD"),
        Symbol::new("USD", "CHF"),
        Symbol::new("AUD", "USD"),
        Symbol::new("NZD", "USD"),
        Symbol::new("EUR", "GBP"),
    ];

    println!(
        "{:<10} | {:<20} | {:<20} | {:<20}",
        "Pair", "V1 Baseline", "V5 Apex Pro", "V6 Hyperion"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    let s_v1 = BacktestService::new(
        adapter.clone(),
        Arc::new(PolaNStrategy::v1_production()),
        RiskProfile::default(),
    );
    let s_v5 = BacktestService::new(
        adapter.clone(),
        Arc::new(PolaNStrategy::v5_apex_pro()),
        RiskProfile::default(),
    );
    let s_v6 = BacktestService::new(
        adapter.clone(),
        Arc::new(PolaNStrategy::v6_hyperion_pro()),
        RiskProfile::default(),
    );

    for sym in test_pairs {
        let r1 = s_v1
            .run_simulation(&sym, Timeframe::H1, is_from, oos_to)
            .await;
        let r5 = s_v5
            .run_simulation(&sym, Timeframe::H1, is_from, oos_to)
            .await;
        let r6 = s_v6
            .run_simulation(&sym, Timeframe::H1, is_from, oos_to)
            .await;

        let str1 = if let Ok(r) = r1 {
            format!(
                "{:>6.1} VP (PF {:.2})",
                r.total_valued_pips, r.profit_factor
            )
        } else {
            "Error".to_string()
        };
        let str5 = if let Ok(r) = r5 {
            format!(
                "{:>6.1} VP (PF {:.2})",
                r.total_valued_pips, r.profit_factor
            )
        } else {
            "Error".to_string()
        };
        let str6 = if let Ok(r) = r6 {
            format!(
                "{:>6.1} VP (PF {:.2})",
                r.total_valued_pips, r.profit_factor
            )
        } else {
            "Error".to_string()
        };

        let sym_name = format!("{}/{}", sym.base, sym.quote);
        println!(
            "{:<10} | {:<20} | {:<20} | {:<20}",
            sym_name, str1, str5, str6
        );
    }

    println!("=========================================================================================\n");
}
