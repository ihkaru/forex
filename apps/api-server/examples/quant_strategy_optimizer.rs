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
        "EURUSD", "EURGBP", "USDCHF", "GBPUSD", "NZDUSD", "AUDUSD", "XAUUSD",
    ];
    let from = Utc.with_ymd_and_hms(2018, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

    println!("==========================================================================");
    println!("  QUANT LAB: STRATEGY OPTIMIZATION MATRIX (2018-2025 • 7 PAIRS)           ");
    println!("==========================================================================");

    for left in [3, 5, 7, 10] {
        for right in [2, 3, 5] {
            for rr in [dec!(1.2), dec!(1.49), dec!(1.8), dec!(2.0)] {
                let mut total_vp = Decimal::ZERO;
                let mut total_trades = 0;
                let mut total_wins = 0;
                let mut total_profit = Decimal::ZERO;
                let mut total_loss = Decimal::ZERO;

                for pair in &pairs {
                    let sym = match *pair {
                        "EURUSD" => Symbol::new("EUR", "USD"),
                        "EURGBP" => Symbol::new("EUR", "GBP"),
                        "USDCHF" => Symbol::new("USD", "CHF"),
                        "GBPUSD" => Symbol::new("GBP", "USD"),
                        "NZDUSD" => Symbol::new("NZD", "USD"),
                        "AUDUSD" => Symbol::new("AUD", "USD"),
                        "XAUUSD" => Symbol::new("XAU", "USD"),
                        _ => unreachable!(),
                    };

                    let strategy = Arc::new(PolaNStrategy::with_params(
                        "TF-PolaN-Opt",
                        left,
                        right,
                        dec!(0.00020),
                        rr,
                    ));
                    let service =
                        BacktestService::new(adapter.clone(), strategy, RiskProfile::default());

                    if let Ok(report) = service.run_simulation(&sym, Timeframe::H1, from, to).await
                    {
                        total_vp += report.total_valued_pips;
                        total_trades += report.total_trades;
                        total_wins += report.winning_trades;
                        total_profit += report.gross_profit_pips;
                        total_loss += report.gross_loss_pips;
                    }
                }

                let wr = if total_trades > 0 {
                    Decimal::from(total_wins) / Decimal::from(total_trades) * dec!(100.0)
                } else {
                    dec!(0.0)
                };
                let pf = if total_loss > Decimal::ZERO {
                    total_profit / total_loss
                } else {
                    dec!(0.0)
                };

                println!(
                    "Swings: ({:>2},{:>1}) | R:R: {:>4} | Trades: {:<3} | WinRate: {:>5.1}% | Total VP: {:>8.1} | PF: {:>4.2}",
                    left, right, rr, total_trades, wr, total_vp, pf
                );
            }
        }
    }
    println!("Matrix run complete.");
}
