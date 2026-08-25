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

    println!("\n=========================================================================================");
    println!(
        "  POLA N V3 RESEARCH: PURE N-PATTERN & INSTITUTIONAL FILTER TUNING (10 YEARS XAUUSD)"
    );
    println!(
        "========================================================================================="
    );
    println!(
        "{:<20} | {:<7} | {:<8} | {:<12} | {:<7} | {:<8} | {:<8}",
        "Configuration", "Trades", "Trd/Bln", "Valued Pips", "WinRate", "PF", "RecFactor"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    // Test 1: Baseline V2 Performance
    let strat_v2 = Arc::new(PolaNStrategy::v2_adaptive());
    let s_v2 = BacktestService::new(adapter.clone(), strat_v2, RiskProfile::default());
    if let Ok(rep) = s_v2.run_simulation(&gold, Timeframe::H1, from, to).await {
        let trd_per_mo = (Decimal::from(rep.total_trades) / dec!(137.0)).round_dp(1);
        println!(
            "★ {:<18} | {:<7} | {:<8} | {:>10.1} VP | {:>6.1}% | {:>8.2} | {:>8.2}",
            "V2 Baseline",
            rep.total_trades,
            trd_per_mo,
            rep.total_valued_pips,
            rep.win_rate_percent,
            rep.profit_factor,
            rep.recovery_factor
        );
    }
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    // Test 2: Multi-parameter search for Pola N v3
    println!(">>> PHASE 2: DEEP FORMATION & GEOMETRIC SEARCH (GOLD 10 YEARS) <<<");
    println!(
        "{:<26} | {:<7} | {:<8} | {:<12} | {:<7} | {:<8} | {:<8}",
        "Configuration", "Trades", "Trd/Bln", "Valued Pips", "WinRate", "PF", "RecFactor"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    let mut best_vp = Decimal::ZERO;
    let mut best_config = String::new();

    for (left, right) in [(4, 3), (4, 2), (5, 3), (3, 2)] {
        for offset in [dec!(0.15), dec!(0.20), dec!(0.25), dec!(0.30)] {
            for rr in [
                dec!(1.08),
                dec!(1.10),
                dec!(1.15),
                dec!(1.20),
                dec!(1.30),
                dec!(1.40),
            ] {
                let label = format!("Test-V3-Sw({left},{right})-Off{offset}-RR{rr}");
                let mut strat = PolaNStrategy::with_params(&label, left, right, dec!(0.00020), rr);
                strat.formation_engine.entry_offset = offset;
                let s =
                    BacktestService::new(adapter.clone(), Arc::new(strat), RiskProfile::default());

                if let Ok(rep) = s.run_simulation(&gold, Timeframe::H1, from, to).await {
                    let trd_per_mo = (Decimal::from(rep.total_trades) / dec!(137.0)).round_dp(1);
                    if rep.total_valued_pips > best_vp && rep.profit_factor >= dec!(1.40) {
                        best_vp = rep.total_valued_pips;
                        best_config = label.clone();
                    }

                    if rep.total_valued_pips >= dec!(8000.0) && rep.profit_factor >= dec!(1.50) {
                        println!(
                            "🔥 {:<24} | {:<7} | {:<8} | {:>10.1} VP | {:>6.1}% | {:>8.2} | {:>8.2}",
                            format!("Sw({left},{right}) Off:{offset} R:{rr}"),
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
    }

    println!("\n🏆 ABSOLUTE BEST CONFIG BY VALUED PIPS: {best_config} with {best_vp:.1} VP!");

    // Test 3: Phase 3 - Filtering Optimization on Sw(4,3) & Sw(4,2)
    println!("\n>>> PHASE 3: SELECTIVE INSTITUTIONAL FILTERS ON GOLD (10 YEARS) <<<");
    println!(
        "{:<35} | {:<7} | {:<8} | {:<12} | {:<7} | {:<8} | {:<8}",
        "Filter Preset", "Trades", "Trd/Bln", "Valued Pips", "WinRate", "PF", "RecFactor"
    );
    println!("───────────────────────────────────────────────────────────────────────────────────────────────────");

    type StrategyFactory = Box<dyn Fn() -> PolaNStrategy>;
    let presets: Vec<(&str, StrategyFactory)> = vec![
        (
            "1. V2 Adaptive Baseline",
            Box::new(PolaNStrategy::v2_adaptive),
        ),
        (
            "2. Pure N-Pattern (No Filters)",
            Box::new(PolaNStrategy::v3_pure_n),
        ),
        (
            "3. Pure N + EMA Fast>Slow Trend",
            Box::new(|| {
                let mut s = PolaNStrategy::v3_pure_n();
                s.enable_ema_filter = true;
                s
            }),
        ),
        (
            "4. Pure N + EMA Trend + EMA Slope",
            Box::new(|| {
                let mut s = PolaNStrategy::v3_pure_n();
                s.enable_ema_filter = true;
                s.enable_slope_filter = true;
                s
            }),
        ),
        (
            "5. Pure N + EMA Trend + Candle Filter",
            Box::new(|| {
                let mut s = PolaNStrategy::v3_pure_n();
                s.enable_ema_filter = true;
                s.enable_candle_filter = true;
                s
            }),
        ),
        (
            "6. V3 Institutional Gold Pro",
            Box::new(PolaNStrategy::v3_gold_pro),
        ),
        (
            "7. V3 Gold Pro (EMA Trend + Slope + Candle)",
            Box::new(|| {
                let mut s = PolaNStrategy::v3_gold_pro();
                s.enable_slope_filter = true;
                s
            }),
        ),
        (
            "8. V3 Gold Pro (Sw 5,3 + EMA Trend + Candle)",
            Box::new(|| {
                let mut s =
                    PolaNStrategy::with_params("TF-PolaN-V3-Sw53", 5, 3, dec!(0.00020), dec!(1.10));
                s.enable_session_filter = false;
                s.enable_candle_filter = true;
                s.enable_ema_filter = true;
                s.enable_slope_filter = false;
                s.enable_rsi_filter = false;
                s
            }),
        ),
        (
            "9. V3 Gold Pro (Sw 5,3 + EMA Trend + Slope + Candle)",
            Box::new(|| {
                let mut s = PolaNStrategy::with_params(
                    "TF-PolaN-V3-Sw53-Slope",
                    5,
                    3,
                    dec!(0.00020),
                    dec!(1.10),
                );
                s.enable_session_filter = false;
                s.enable_candle_filter = true;
                s.enable_ema_filter = true;
                s.enable_slope_filter = true;
                s.enable_rsi_filter = false;
                s
            }),
        ),
        (
            "10. V3 Gold Pro (Sw 5,3 + Slope + RR 1.15)",
            Box::new(|| {
                let mut s = PolaNStrategy::with_params(
                    "TF-PolaN-V3-RR115",
                    5,
                    3,
                    dec!(0.00020),
                    dec!(1.15),
                );
                s.enable_session_filter = false;
                s.enable_candle_filter = true;
                s.enable_ema_filter = true;
                s.enable_slope_filter = true;
                s.enable_rsi_filter = false;
                s
            }),
        ),
        (
            "11. V3 Gold Pro (Sw 5,3 + Slope + RR 1.20)",
            Box::new(|| {
                let mut s = PolaNStrategy::with_params(
                    "TF-PolaN-V3-RR120",
                    5,
                    3,
                    dec!(0.00020),
                    dec!(1.20),
                );
                s.enable_session_filter = false;
                s.enable_candle_filter = true;
                s.enable_ema_filter = true;
                s.enable_slope_filter = true;
                s.enable_rsi_filter = false;
                s
            }),
        ),
    ];

    for (name, factory) in presets {
        let strategy = Arc::new(factory());
        let s = BacktestService::new(adapter.clone(), strategy, RiskProfile::default());
        if let Ok(rep) = s.run_simulation(&gold, Timeframe::H1, from, to).await {
            let trd_per_mo = (Decimal::from(rep.total_trades) / dec!(137.0)).round_dp(1);
            println!(
                "{:<35} | {:<7} | {:<8} | {:>10.1} VP | {:>6.1}% | {:>8.2} | {:>8.2}",
                name,
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
