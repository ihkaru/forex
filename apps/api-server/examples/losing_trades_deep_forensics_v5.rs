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
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let gold = Symbol::new("XAU", "USD");
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    println!("\n=========================================================================================");
    println!(
        "  POLA N V5 APEX PRO: DEEP LOSS FORENSICS & FAILURE MODE BREAKDOWN (10 YEARS XAUUSD)"
    );
    println!(
        "========================================================================================="
    );

    let strat_v5 = Arc::new(PolaNStrategy::v5_apex_pro());
    let service = BacktestService::new(adapter.clone(), strat_v5, RiskProfile::default());

    let sim_result = service
        .run_simulation_detailed(&gold, Timeframe::H1, from, to)
        .await
        .expect("Gagal menjalankan backtest V5");

    println!("Total Trades       : {}", sim_result.report.total_trades);
    println!("Winning Trades     : {}", sim_result.report.winning_trades);
    println!("Losing Trades      : {}", sim_result.report.losing_trades);
    println!(
        "Win Rate           : {:.1}%",
        sim_result.report.win_rate_percent
    );
    println!(
        "Total Valued Pips  : {:.1} VP",
        sim_result.report.total_valued_pips
    );
    println!(
        "Profit Factor      : {:.2}",
        sim_result.report.profit_factor
    );
    println!(
        "Max Drawdown       : {:.1} pips",
        sim_result.report.max_drawdown_pips
    );
    println!(
        "Recovery Factor    : {:.2}",
        sim_result.report.recovery_factor
    );

    let candles = adapter
        .get_historical_candles(&gold, Timeframe::H1, from, to)
        .await
        .expect("Gagal memuat candle");

    let mut near_tp_losses = 0; // Price went >= 50% toward TP then hit SL
    let mut immediate_fakeouts = 0; // Stopped out within 1-2 bars
    let mut stagnation_losses = 0; // Held > 12 bars before SL
    let mut asian_session_losses = 0; // Hour 00-06 UTC
    let mut london_ny_losses = 0; // Hour 07-18 UTC
    let mut late_ny_losses = 0; // Hour 19-23 UTC
    let mut friday_losses = 0;
    let mut counter_ema100_losses = 0;
    let mut low_volatility_losses = 0;

    let mut consecutive_losses = 0;
    let mut max_consecutive_losses = 0;

    println!("\n--- SAMPLE LOSING TRADES AUDIT (FIRST 12) ---");
    let mut sample_count = 0;

    for (idx, trade) in sim_result.trades.iter().enumerate() {
        let pnl = trade.realized_pnl.unwrap_or(Decimal::ZERO);
        if pnl <= Decimal::ZERO {
            consecutive_losses += 1;
            if consecutive_losses > max_consecutive_losses {
                max_consecutive_losses = consecutive_losses;
            }

            use chrono::Timelike;
            let hour = trade.open_time.hour();
            let weekday = trade.open_time.weekday();

            if weekday == chrono::Weekday::Fri && hour >= 16 {
                friday_losses += 1;
            }

            if hour <= 6 {
                asian_session_losses += 1;
            } else if hour <= 18 {
                london_ny_losses += 1;
            } else {
                late_ny_losses += 1;
            }

            let close_time = trade.close_time.unwrap_or(trade.open_time);
            let duration_hours = (close_time - trade.open_time).num_hours();
            if duration_hours <= 2 {
                immediate_fakeouts += 1;
            }
            if duration_hours > 12 {
                stagnation_losses += 1;
            }

            // Cari bars di antara open_time dan close_time untuk hitung Peak MFE
            let trade_candles: Vec<_> = candles
                .iter()
                .filter(|c| c.timestamp >= trade.open_time && c.timestamp <= close_time)
                .collect();

            let target_dist = (trade.take_profit - trade.open_price).abs();
            let mut max_fav = Decimal::ZERO;

            for c in &trade_candles {
                let fav = match trade.action {
                    domain::models::SignalAction::BuyStop
                    | domain::models::SignalAction::BuyLimit => c.high - trade.open_price,
                    domain::models::SignalAction::SellStop
                    | domain::models::SignalAction::SellLimit => trade.open_price - c.low,
                    _ => Decimal::ZERO,
                };
                if fav > max_fav {
                    max_fav = fav;
                }
            }

            let mfe_ratio = if target_dist > Decimal::ZERO {
                max_fav / target_dist
            } else {
                Decimal::ZERO
            };

            if mfe_ratio >= dec!(0.50) {
                near_tp_losses += 1;
            }

            // Cek kondisi candle saat open
            let candle_idx = candles.iter().position(|c| c.timestamp == trade.open_time);
            if let Some(c_idx) = candle_idx {
                if c_idx >= 100 {
                    let sub_candles = &candles[..=c_idx];
                    if let Some(ema100) =
                        domain::models::pola_n::detector::calculate_ema(sub_candles, 100)
                    {
                        match trade.action {
                            domain::models::SignalAction::BuyStop
                            | domain::models::SignalAction::BuyLimit => {
                                if trade.open_price < ema100 {
                                    counter_ema100_losses += 1;
                                }
                            }
                            domain::models::SignalAction::SellStop
                            | domain::models::SignalAction::SellLimit => {
                                if trade.open_price > ema100 {
                                    counter_ema100_losses += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    if let (Some(atr14), Some(atr30)) = (
                        domain::models::pola_n::detector::calculate_atr(sub_candles, 14),
                        domain::models::pola_n::detector::calculate_atr(sub_candles, 30),
                    ) {
                        if atr14 < (atr30 * dec!(0.90)) {
                            low_volatility_losses += 1;
                        }
                    }
                }
            }

            if sample_count < 12 {
                sample_count += 1;
                println!(
                    "Loss #{:<2} [Trade #{:<4}]: {:?} @ {} | Open: {} ({:?}) | Close: {} | Dur: {:>2}h | MFE: {:>5.1}% | PnL: {:>6.1}",
                    sample_count,
                    idx + 1,
                    trade.action,
                    trade.open_price,
                    trade.open_time.format("%Y-%m-%d %H:%M"),
                    weekday,
                    close_time.format("%Y-%m-%d %H:%M"),
                    duration_hours,
                    mfe_ratio * dec!(100.0),
                    pnl
                );
            }
        } else {
            consecutive_losses = 0;
        }
    }

    let total_losses = sim_result.report.losing_trades as f64;
    println!("\n=========================================================================================");
    println!(
        "📊 FORENSIK DISTRIBUSI KEKALAHAN V5 ({} LOSING TRADES):",
        total_losses
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "1. Near-TP Reversals (MFE >= 50% lalu kena SL) : {:>4} trades ({:.1}%)",
        near_tp_losses,
        (near_tp_losses as f64 / total_losses) * 100.0
    );
    println!(
        "2. Immediate Fakeouts (SL dalam <= 2 jam)       : {:>4} trades ({:.1}%)",
        immediate_fakeouts,
        (immediate_fakeouts as f64 / total_losses) * 100.0
    );
    println!(
        "3. Stagnasi Lambat (Floating > 12 jam)          : {:>4} trades ({:.1}%)",
        stagnation_losses,
        (stagnation_losses as f64 / total_losses) * 100.0
    );
    println!(
        "4. Melawan Tren Macro 100 EMA                   : {:>4} trades ({:.1}%)",
        counter_ema100_losses,
        (counter_ema100_losses as f64 / total_losses) * 100.0
    );
    println!(
        "5. Low Volatility Squeeze (ATR14 < 0.9 ATR30)   : {:>4} trades ({:.1}%)",
        low_volatility_losses,
        (low_volatility_losses as f64 / total_losses) * 100.0
    );
    println!(
        "6. Friday Weekend Rollover (>16:00 UTC)         : {:>4} trades ({:.1}%)",
        friday_losses,
        (friday_losses as f64 / total_losses) * 100.0
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!("🌍 DISTRIBUSI SESI WAKTU KEKALAHAN:");
    println!(
        "• Sesi Asia / Mati Likuiditas (00:00 - 06:00 UTC) : {:>4} trades ({:.1}%)",
        asian_session_losses,
        (asian_session_losses as f64 / total_losses) * 100.0
    );
    println!(
        "• Sesi London & NY Utama (07:00 - 18:00 UTC)      : {:>4} trades ({:.1}%)",
        london_ny_losses,
        (london_ny_losses as f64 / total_losses) * 100.0
    );
    println!(
        "• Sesi Late NY / Rollover (19:00 - 23:00 UTC)     : {:>4} trades ({:.1}%)",
        late_ny_losses,
        (late_ny_losses as f64 / total_losses) * 100.0
    );
    println!("=========================================================================================\n");
}
