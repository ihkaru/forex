#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    clippy::all
)]
use api_server::state::RealHistoricalMarketAdapter;
use application::services::BacktestService;
use chrono::{Datelike, TimeZone, Utc};
use domain::models::{PolaNStrategy, RiskProfile, Symbol, Timeframe};
use domain::ports::MarketDataPort;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let gold = Symbol::new("XAU", "USD");
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    println!("\n=========================================================================================");
    println!("  POLA N V7 INSTITUTIONAL CIRCUIT-BREAKER SIMULATOR (TARGET 100% POSITIVE MONTHS)");
    println!(
        "========================================================================================="
    );

    let strat = Arc::new(PolaNStrategy::v6_hyperion_pro());
    let service = BacktestService::new(adapter.clone(), strat, RiskProfile::default());

    let sim_result = service
        .run_simulation_detailed(&gold, Timeframe::H1, from, to)
        .await
        .expect("Gagal simulasi V6");

    println!("Total Raw Trades: {}", sim_result.trades.len());

    // Uji berbagai circuit breaker bulanan:
    // Max Loss per month: -50 pips, -75 pips, -100 pips, -150 pips
    // Profit Lock: +300 VP, +400 VP, None

    for max_monthly_loss in [dec!(50.0), dec!(75.0), dec!(100.0), dec!(150.0)] {
        for profit_lock_vp in [None, Some(dec!(300.0)), Some(dec!(500.0))] {
            let lock_str = match profit_lock_vp {
                Some(p) => format!("Lock+{p}VP"),
                None => "NoLock".to_string(),
            };

            let mut monthly_pnl: BTreeMap<String, Decimal> = BTreeMap::new();
            let mut monthly_trades_count: BTreeMap<String, usize> = BTreeMap::new();
            let mut accepted_trades = 0;
            let mut total_realized_vp = Decimal::ZERO;

            for trade in &sim_result.trades {
                let m = trade.open_time.format("%Y-%m").to_string();
                let current_month_pnl = monthly_pnl.entry(m.clone()).or_default();
                let current_trades = monthly_trades_count.entry(m.clone()).or_default();

                // Cek apakah circuit breaker aktif bulan ini
                if *current_month_pnl <= -max_monthly_loss {
                    continue; // Skip trade karena bulan ini sudah kena batas proteksi modal!
                }

                if let Some(target_lock) = profit_lock_vp {
                    let current_vp = *current_month_pnl * dec!(0.50);
                    if current_vp >= target_lock && *current_trades >= 5 {
                        continue; // Target TF 300+ VP sudah tercapai dan >= 5 trade settled, kunci profit!
                    }
                }

                let pnl = trade.realized_pnl.unwrap_or(Decimal::ZERO);
                *current_month_pnl += pnl;
                *current_trades += 1;
                accepted_trades += 1;
                total_realized_vp += pnl * dec!(0.50);
            }

            let mut pos_months = 0;
            let mut neg_months = 0;
            let mut zero_months = 0;
            let mut total_tf_qualified_months = 0;

            for (_m, pnl) in &monthly_pnl {
                let vp = *pnl * dec!(0.50);
                if vp > Decimal::ZERO {
                    pos_months += 1;
                } else if vp < Decimal::ZERO {
                    neg_months += 1;
                } else {
                    zero_months += 1;
                }

                if vp >= dec!(300.0) {
                    total_tf_qualified_months += 1;
                }
            }

            let total_months = monthly_pnl.len() as f64;
            let pos_ratio = (pos_months as f64 / total_months) * 100.0;

            println!(
                "Config: MaxLoss -{:>5.1} pips | {:<12} -> {:>4} Trades | Total VP: {:>8.1} | Pos: {:>3} ({:>5.1}%) | Neg: {:>2} | TF Qual: {:>2} bln",
                max_monthly_loss,
                lock_str,
                accepted_trades,
                total_realized_vp,
                pos_months,
                pos_ratio,
                neg_months,
                total_tf_qualified_months
            );
        }
    }
}
