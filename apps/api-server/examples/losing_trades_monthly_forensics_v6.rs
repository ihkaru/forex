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
    println!("  POLA N V6 MONTHLY FORENSICS: IDENTIFYING ALL NEGATIVE VP MONTHS (10 YEARS XAUUSD)");
    println!(
        "========================================================================================="
    );

    let strat_v6 = Arc::new(PolaNStrategy::v6_hyperion_pro());
    let service = BacktestService::new(adapter.clone(), strat_v6, RiskProfile::default());

    let sim_result = service
        .run_simulation_detailed(&gold, Timeframe::H1, from, to)
        .await
        .expect("Gagal menjalankan backtest V6");

    println!("Total Trades: {}", sim_result.report.total_trades);
    println!(
        "Total Valued Pips: {:.1} VP",
        sim_result.report.total_valued_pips
    );
    println!("Profit Factor: {:.2}", sim_result.report.profit_factor);

    // Kelompokkan trade per bulan: "YYYY-MM"
    let mut monthly_trades: BTreeMap<String, Vec<&domain::models::Order>> = BTreeMap::new();

    for trade in &sim_result.trades {
        let month_key = trade.open_time.format("%Y-%m").to_string();
        monthly_trades.entry(month_key).or_default().push(trade);
    }

    let mut negative_months = Vec::new();
    let mut positive_months = 0;
    let mut zero_months = 0;

    println!("\n--- RINCIAN BULAN-BULAN DENGAN VALUED PIPS NEGATIF DI V6 ---");
    println!(
        "{:<8} | {:<7} | {:<5} | {:<5} | {:<8} | {:<12} | {:<8}",
        "Bulan", "Trades", "Wins", "Loss", "WinRate", "Valued Pips", "Loss Cause"
    );
    println!("─────────────────────────────────────────────────────────────────────────────");

    let candles = adapter
        .get_historical_candles(&gold, Timeframe::H1, from, to)
        .await
        .expect("Gagal memuat candle");

    for (month, trades) in &monthly_trades {
        let mut total_pnl_pips = Decimal::ZERO;
        let mut wins = 0;
        let mut losses = 0;

        for t in trades {
            let pnl = t.realized_pnl.unwrap_or(Decimal::ZERO);
            total_pnl_pips += pnl;
            if pnl > Decimal::ZERO {
                wins += 1;
            } else {
                losses += 1;
            }
        }

        // Valued Pips untuk Gold (Tier 4) = Pips * 0.50
        let valued_pips = total_pnl_pips * dec!(0.50);
        let win_rate = if !trades.is_empty() {
            (Decimal::from(wins) / Decimal::from(trades.len())) * dec!(100.0)
        } else {
            Decimal::ZERO
        };

        if valued_pips < Decimal::ZERO {
            // Analisis mengapa bulan ini negatif
            let sample_time = trades[0].open_time;
            let c_idx_opt = candles.iter().position(|c| c.timestamp >= sample_time);

            let mut regime_desc = "Ranging / False Breakouts".to_string();
            if let Some(idx) = c_idx_opt {
                if idx >= 30 {
                    if let (Some(atr14), Some(atr30)) = (
                        domain::models::pola_n::detector::calculate_atr(&candles[..idx], 14),
                        domain::models::pola_n::detector::calculate_atr(&candles[..idx], 30),
                    ) {
                        if atr14 < atr30 {
                            regime_desc = "Low Volatility Squeeze (Chop)".to_string();
                        } else {
                            regime_desc = "High Volatility Choppy Whipsaw".to_string();
                        }
                    }
                }
            }

            negative_months.push((
                month.clone(),
                trades.len(),
                wins,
                losses,
                win_rate,
                valued_pips,
                regime_desc.clone(),
            ));
            println!(
                "{:<8} | {:<7} | {:<5} | {:<5} | {:>6.1}% | {:>10.1} VP | {}",
                month,
                trades.len(),
                wins,
                losses,
                win_rate,
                valued_pips,
                regime_desc
            );
        } else if valued_pips > Decimal::ZERO {
            positive_months += 1;
        } else {
            zero_months += 1;
        }
    }

    let total_months = monthly_trades.len();
    println!("\n=========================================================================================");
    println!(
        "📊 SUMMARY BULANAN V6 (TOTAL {} BULAN / 10 TAHUN):",
        total_months
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "• Bulan Profit Positif (+VP) : {:>3} bulan ({:.1}%)",
        positive_months,
        (positive_months as f64 / total_months as f64) * 100.0
    );
    println!(
        "• Bulan Rugi Negatif (-VP)   : {:>3} bulan ({:.1}%)",
        negative_months.len(),
        (negative_months.len() as f64 / total_months as f64) * 100.0
    );
    println!("• Bulan Netral (0 VP)        : {:>3} bulan", zero_months);
    println!("=========================================================================================\n");
}
