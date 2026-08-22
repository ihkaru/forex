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
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

    println!("==========================================================================");
    println!("  TESTING INSTITUTIONAL PENDING ENTRY MODELS (7 PAIRS • 10 YEARS)         ");
    println!("==========================================================================");

    for rr in [dec!(1.49), dec!(1.8), dec!(2.0), dec!(2.5)] {
        println!("\n>>> EVALUATING SWINGS (4,3) WITH R:R = {} <<<", rr);
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
                4,
                3,
                dec!(0.00020),
                rr,
            ));
            let service = BacktestService::new(adapter.clone(), strategy, RiskProfile::default());

            if let Ok(report) = service.run_simulation(&sym, Timeframe::H1, from, to).await {
                let wr = if report.total_trades > 0 {
                    Decimal::from(report.winning_trades) / Decimal::from(report.total_trades)
                        * dec!(100.0)
                } else {
                    dec!(0.0)
                };
                println!(
                    "Pair: {:<6} | Trades: {:<3} | WinRate: {:>5.1}% | ProfitFactor: {:>4.2} | Valued Pips: {:>8.1}",
                    pair, report.total_trades, wr, report.profit_factor, report.total_valued_pips
                );
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
            "OVERALL: Trades: {:<4} | WinRate: {:>5.1}% | Total VP: {:>9.1} | PF: {:>4.2}",
            total_trades, wr, total_vp, pf
        );
    }
}
