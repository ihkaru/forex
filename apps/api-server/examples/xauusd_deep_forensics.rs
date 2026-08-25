#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    clippy::all
)]
use api_server::state::RealHistoricalMarketAdapter;
use application::services::BacktestService;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use domain::models::{PolaNStrategy, RiskProfile, SignalAction, Symbol, TfPairSpec, Timeframe};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let sym = Symbol::new("XAU", "USD");
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let strategy = Arc::new(PolaNStrategy::with_params(
        "TF-PolaN-XAUUSD-Forensics",
        5,
        3,
        dec!(0.00020),
        dec!(1.3),
    ));

    let service = BacktestService::new(adapter.clone(), strategy, RiskProfile::default());
    let detailed = service
        .run_simulation_detailed(&sym, Timeframe::H1, from, to)
        .await
        .expect("Backtest XAUUSD gagal");

    let report = &detailed.report;
    let trades = &detailed.trades;
    let spec = TfPairSpec::from_symbol(&sym);

    println!("================================================================================");
    println!("  🔬 DEEP QUANTITATIVE FORENSIC REPORT: XAUUSD (GOLD H1)                        ");
    println!("================================================================================");
    println!("  Total Trades       : {}", report.total_trades);
    println!(
        "  Winning Trades     : {} ({:.1}%)",
        report.winning_trades, report.win_rate_percent
    );
    println!("  Losing Trades      : {}", report.losing_trades);
    println!(
        "  Breakeven Exits    : {}",
        report.total_trades - report.winning_trades - report.losing_trades
    );
    println!(
        "  Gross Profit       : +{:.1} pips",
        report.gross_profit_pips
    );
    println!("  Gross Loss         : -{:.1} pips", report.gross_loss_pips);
    println!("  Net Profit         : {:+.1} pips", report.total_raw_pips);
    println!("  Total Valued Pips  : {:+.1} VP", report.total_valued_pips);
    println!("  Profit Factor      : {:.2}", report.profit_factor);
    println!(
        "  Max Drawdown       : -{:.1} pips",
        report.max_drawdown_pips
    );
    println!("  Recovery Factor    : {:.2}", report.recovery_factor);
    println!("================================================================================\n");

    // 1. Breakdown Berdasarkan Tahun (Market Regimes & Periods)
    println!("📅 1. BREAKDOWN PERFORMA PER-TAHUN (PERIOD REGIMES):");
    println!("Tahun | Trades | Wins | Losses | BE | WinRate | Net Pips    | Valued Pips | PF");
    println!("{}", "─".repeat(78));

    let mut year_map: BTreeMap<i32, Vec<&domain::models::Order>> = BTreeMap::new();
    for trade in trades {
        let year = trade.open_time.year();
        year_map.entry(year).or_default().push(trade);
    }

    for (year, year_trades) in &year_map {
        let mut y_wins = 0;
        let mut y_losses = 0;
        let mut y_be = 0;
        let mut y_gp = Decimal::ZERO;
        let mut y_gl = Decimal::ZERO;
        let mut y_net = Decimal::ZERO;

        for t in year_trades {
            if let Some(pnl) = t.realized_pnl {
                y_net += pnl;
                if pnl > Decimal::ZERO {
                    y_wins += 1;
                    y_gp += pnl;
                } else if pnl < Decimal::ZERO {
                    y_losses += 1;
                    y_gl += pnl.abs();
                } else {
                    y_be += 1;
                }
            }
        }

        let total = year_trades.len();
        let wr = if total > 0 {
            Decimal::from(y_wins) / Decimal::from(total) * dec!(100.0)
        } else {
            dec!(0.0)
        };
        let pf = if y_gl > Decimal::ZERO {
            y_gp / y_gl
        } else {
            dec!(99.9)
        };
        let vp = spec.pips_to_valued_pips(y_net);

        println!(
            "{:<5} | {:<6} | {:<4} | {:<6} | {:<2} | {:<6.1}% | {:<+11.1} | {:<+11.1} | {:.2}",
            year, total, y_wins, y_losses, y_be, wr, y_net, vp, pf
        );
    }

    // 2. Breakdown Long (Bullish N) vs Short (Bearish N)
    println!("\n⚖️ 2. BREAKDOWN LONG VS SHORT:");
    let mut long_trades = 0;
    let mut long_wins = 0;
    let mut long_losses = 0;
    let mut long_pnl = Decimal::ZERO;
    let mut long_gp = Decimal::ZERO;
    let mut long_gl = Decimal::ZERO;

    let mut short_trades = 0;
    let mut short_wins = 0;
    let mut short_losses = 0;
    let mut short_pnl = Decimal::ZERO;
    let mut short_gp = Decimal::ZERO;
    let mut short_gl = Decimal::ZERO;

    for t in trades {
        let is_long = matches!(t.action, SignalAction::BuyLimit | SignalAction::BuyStop);
        if let Some(pnl) = t.realized_pnl {
            if is_long {
                long_trades += 1;
                long_pnl += pnl;
                if pnl > Decimal::ZERO {
                    long_wins += 1;
                    long_gp += pnl;
                } else if pnl < Decimal::ZERO {
                    long_losses += 1;
                    long_gl += pnl.abs();
                }
            } else {
                short_trades += 1;
                short_pnl += pnl;
                if pnl > Decimal::ZERO {
                    short_wins += 1;
                    short_gp += pnl;
                } else if pnl < Decimal::ZERO {
                    short_losses += 1;
                    short_gl += pnl.abs();
                }
            }
        }
    }

    let long_pf = if long_gl > Decimal::ZERO {
        long_gp / long_gl
    } else {
        dec!(0.0)
    };
    let short_pf = if short_gl > Decimal::ZERO {
        short_gp / short_gl
    } else {
        dec!(0.0)
    };
    println!(
        "  LONG  (BuyStop/BuyLimit)  : {} trades | {} wins ({} losses) | Net: {:+.1} pips | PF: {:.2}",
        long_trades, long_wins, long_losses, long_pnl, long_pf
    );
    println!(
        "  SHORT (SellStop/SellLimit): {} trades | {} wins ({} losses) | Net: {:+.1} pips | PF: {:.2}",
        short_trades, short_wins, short_losses, short_pnl, short_pf
    );

    // 3. Breakdown Jam Eksekusi (Session Timing Analysis)
    println!("\n⏰ 3. BREAKDOWN JAM EKSEKUSI (UTC):");
    let mut hour_map: BTreeMap<u32, (usize, usize, Decimal, Decimal)> = BTreeMap::new();
    for t in trades {
        let hour = t.open_time.hour();
        let entry = hour_map
            .entry(hour)
            .or_insert((0, 0, Decimal::ZERO, Decimal::ZERO));
        entry.0 += 1;
        if let Some(pnl) = t.realized_pnl {
            if pnl > Decimal::ZERO {
                entry.1 += 1;
                entry.2 += pnl;
            } else if pnl < Decimal::ZERO {
                entry.3 += pnl.abs();
            }
        }
    }

    println!("Jam UTC | Trades | Wins | WinRate | Gross Profit | Gross Loss | PF");
    println!("{}", "─".repeat(68));
    for (hour, (trds, wins, gp, gl)) in &hour_map {
        let wr = if *trds > 0 {
            Decimal::from(*wins) / Decimal::from(*trds) * dec!(100.0)
        } else {
            dec!(0.0)
        };
        let pf = if *gl > Decimal::ZERO {
            *gp / *gl
        } else {
            dec!(0.0)
        };
        let session_name = match hour {
            6..=8 => "Frankfurt Open",
            9..=12 => "London Morning",
            13..=16 => "London/NY Overlap",
            17..=20 => "NY Afternoon",
            _ => "Asian / Off-Hours",
        };
        println!(
            "{:02}:00   | {:<6} | {:<4} | {:<6.1}% | {:<12.1} | {:<10.1} | {:<4.2} ({})",
            hour, trds, wins, wr, gp, gl, pf, session_name
        );
    }

    // 4. Forensik Losing Trades Terbesar (Top 5 Worst Losses)
    println!("\n🔍 4. FORENSIK 5 TRADE RUGI TERBESAR (ROOT CAUSE ANALYSIS):");
    let mut losing_trades: Vec<&domain::models::Order> = trades
        .iter()
        .filter(|t| t.realized_pnl.unwrap_or(Decimal::ZERO) < Decimal::ZERO)
        .collect();
    losing_trades.sort_by(|a, b| {
        a.realized_pnl
            .unwrap_or(Decimal::ZERO)
            .cmp(&b.realized_pnl.unwrap_or(Decimal::ZERO))
    });

    for (i, t) in losing_trades.iter().take(5).enumerate() {
        let pnl = t.realized_pnl.unwrap_or(Decimal::ZERO);
        let duration = (t.close_time.unwrap_or(t.open_time) - t.open_time).num_hours();
        println!(
            "  [Loss #{}] {} | Action: {:?} | Open: {} | PnL: {:+.1} pips | Held: {}h | SL: {} | TP: {}",
            i + 1,
            sym,
            t.action,
            t.open_time.format("%Y-%m-%d %H:%M"),
            pnl,
            duration,
            t.stop_loss,
            t.take_profit
        );
    }

    // 5. Forensik Winning Trades Terbesar (Top 5 Best Wins)
    println!("\n🎯 5. FORENSIK 5 TRADE MENANG TERBESAR (WIN PATTERN ANALYSIS):");
    let mut win_trades: Vec<&domain::models::Order> = trades
        .iter()
        .filter(|t| t.realized_pnl.unwrap_or(Decimal::ZERO) > Decimal::ZERO)
        .collect();
    win_trades.sort_by(|a, b| {
        b.realized_pnl
            .unwrap_or(Decimal::ZERO)
            .cmp(&a.realized_pnl.unwrap_or(Decimal::ZERO))
    });

    for (i, t) in win_trades.iter().take(5).enumerate() {
        let pnl = t.realized_pnl.unwrap_or(Decimal::ZERO);
        let duration = (t.close_time.unwrap_or(t.open_time) - t.open_time).num_hours();
        println!(
            "  [Win #{}] {} | Action: {:?} | Open: {} | PnL: {:+.1} pips | Held: {}h | Entry: {} | TP: {}",
            i + 1,
            sym,
            t.action,
            t.open_time.format("%Y-%m-%d %H:%M"),
            pnl,
            duration,
            t.open_price,
            t.take_profit
        );
    }
}
