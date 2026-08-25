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
    let pairs = [
        "EURUSD", "EURGBP", "USDCHF", "GBPUSD", "NZDUSD", "AUDUSD", "XAUUSD", "USDJPY",
    ];
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let total_months = dec!(120.0);

    println!("\n▶ EVALUASI DETAIL PER-PAIR (Swing 5,3 | R:R 1.3):");
    let pair_header = format!(
        "{:<8} | {:<7} | {:<8} | {:<12} | {:<7} | {:<8}",
        "Pair", "Trades", "Trd/Bln", "Total VP", "WinRate", "PF"
    );
    println!("{pair_header}");
    println!("{}", "─".repeat(60));

    let mut total_vp = Decimal::ZERO;
    let mut total_trades = 0;
    let mut total_wins = 0;
    let mut total_gross_profit = Decimal::ZERO;
    let mut total_gross_loss = Decimal::ZERO;

    for pair in &pairs {
        let sym = match *pair {
            "EURUSD" => Symbol::new("EUR", "USD"),
            "EURGBP" => Symbol::new("EUR", "GBP"),
            "USDCHF" => Symbol::new("USD", "CHF"),
            "GBPUSD" => Symbol::new("GBP", "USD"),
            "NZDUSD" => Symbol::new("NZD", "USD"),
            "AUDUSD" => Symbol::new("AUD", "USD"),
            "XAUUSD" => Symbol::new("XAU", "USD"),
            "USDJPY" => Symbol::new("USD", "JPY"),
            _ => unreachable!(),
        };

        let strategy = Arc::new(PolaNStrategy::with_params(
            "TF-PolaN-Lab",
            5,
            3,
            dec!(0.00020),
            dec!(1.3),
        ));
        let service = BacktestService::new(adapter.clone(), strategy, RiskProfile::default());

        if let Ok(report) = service.run_simulation(&sym, Timeframe::H1, from, to).await {
            let pair_wr = if report.total_trades > 0 {
                Decimal::from(report.winning_trades) / Decimal::from(report.total_trades)
                    * dec!(100.0)
            } else {
                dec!(0.0)
            };
            let pair_pf = if report.gross_loss_pips > Decimal::ZERO {
                report.gross_profit_pips / report.gross_loss_pips
            } else {
                dec!(0.0)
            };
            let pair_trd_mo = Decimal::from(report.total_trades) / total_months;

            println!(
                "{:<8} | {:<7} | {:<8.1} | {:<+12.1} | {:<6.1}% | {:<8.2}",
                pair, report.total_trades, pair_trd_mo, report.total_valued_pips, pair_wr, pair_pf
            );

            total_vp += report.total_valued_pips;
            total_trades += report.total_trades;
            total_wins += report.winning_trades;
            total_gross_profit += report.gross_profit_pips;
            total_gross_loss += report.gross_loss_pips;
        }
    }

    let wr = if total_trades > 0 {
        Decimal::from(total_wins) / Decimal::from(total_trades) * dec!(100.0)
    } else {
        dec!(0.0)
    };
    let pf = if total_gross_loss > Decimal::ZERO {
        total_gross_profit / total_gross_loss
    } else {
        dec!(0.0)
    };
    let trades_per_month = Decimal::from(total_trades) / total_months;
    println!("{}", "─".repeat(60));
    println!(
        "{:<8} | {:<7} | {:<8.1} | {:<+12.1} | {:<6.1}% | {:<8.2}",
        "TOTAL", total_trades, trades_per_month, total_vp, wr, pf
    );
}
