use api_server::state::RealHistoricalMarketAdapter;
use application::services::BacktestService;
use chrono::{TimeZone, Utc};
use domain::models::{
    Candle, Order, PolaNStrategy, RiskProfile, SignalAction, Symbol, TfPairSpec, Timeframe,
};
use domain::ports::MarketDataPort;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[allow(dead_code)]
struct LossDiagnostics {
    order: Order,
    setup_candles: Vec<Candle>,
    trade_candles: Vec<Candle>,
    max_favorable_pips: Decimal,
    target_tp_pips: Decimal,
    risk_sl_pips: Decimal,
    mfe_pct_of_tp: Decimal,
    duration_bars: usize,
}

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let pairs = ["EURUSD", "GBPUSD", "EURGBP", "AUDUSD", "USDCHF"];
    let from = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║     🔬 FORENSIC POST-MORTEM DIAGNOSTIC: LOSING TRADES & CANDLE ANATOMY       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    let strategy = Arc::new(PolaNStrategy::default());

    for pair_str in pairs {
        let sym = match pair_str {
            "EURUSD" => Symbol::new("EUR", "USD"),
            "GBPUSD" => Symbol::new("GBP", "USD"),
            "EURGBP" => Symbol::new("EUR", "GBP"),
            "AUDUSD" => Symbol::new("AUD", "USD"),
            "USDCHF" => Symbol::new("USD", "CHF"),
            _ => unreachable!(),
        };

        let spec = TfPairSpec::from_symbol(&sym);
        let all_candles = adapter
            .get_historical_candles(&sym, Timeframe::H1, from, to)
            .await
            .unwrap();

        let service =
            BacktestService::new(adapter.clone(), strategy.clone(), RiskProfile::default());
        let detailed = service
            .run_simulation_detailed(&sym, Timeframe::H1, from, to)
            .await
            .unwrap();

        let losing_trades: Vec<&Order> = detailed
            .trades
            .iter()
            .filter(|t| t.realized_pnl.unwrap_or(Decimal::ZERO) < Decimal::ZERO)
            .collect();

        if losing_trades.is_empty() {
            continue;
        }

        println!("══════════════════════════════════════════════════════════════════════════════");
        println!(
            "🔍 PAIR: {} | Total Trades: {} | Losing Trades: {} ({:.1}%)",
            pair_str,
            detailed.trades.len(),
            losing_trades.len(),
            (Decimal::from(losing_trades.len()) / Decimal::from(detailed.trades.len().max(1)))
                * dec!(100.0)
        );
        println!("══════════════════════════════════════════════════════════════════════════════");

        let mut diagnostics: Vec<LossDiagnostics> = Vec::new();

        for loss in &losing_trades {
            let open_t = loss.open_time;
            let close_t = loss.close_time.unwrap_or(open_t);

            // Cari index candle
            let open_idx = all_candles
                .iter()
                .position(|c| c.timestamp >= open_t)
                .unwrap_or(0);
            let close_idx = all_candles
                .iter()
                .position(|c| c.timestamp >= close_t)
                .unwrap_or(open_idx);

            let setup_start = open_idx.saturating_sub(10);
            let setup_candles = all_candles[setup_start..open_idx].to_vec();
            let trade_candles = if close_idx >= open_idx && close_idx < all_candles.len() {
                all_candles[open_idx..=close_idx].to_vec()
            } else {
                Vec::new()
            };

            let entry = loss.open_price;
            let sl = loss.stop_loss;
            let tp = loss.take_profit;

            let target_tp_pips = spec.price_diff_to_pips((tp - entry).abs());
            let risk_sl_pips = spec.price_diff_to_pips((entry - sl).abs());

            let mut max_fav_pips = Decimal::ZERO;
            for tc in &trade_candles {
                let fav = match loss.action {
                    SignalAction::BuyLimit | SignalAction::BuyStop => {
                        if tc.high > entry {
                            spec.price_diff_to_pips(tc.high - entry)
                        } else {
                            Decimal::ZERO
                        }
                    }
                    SignalAction::SellLimit | SignalAction::SellStop => {
                        if tc.low < entry {
                            spec.price_diff_to_pips(entry - tc.low)
                        } else {
                            Decimal::ZERO
                        }
                    }
                    _ => Decimal::ZERO,
                };
                if fav > max_fav_pips {
                    max_fav_pips = fav;
                }
            }

            let mfe_pct_of_tp = if target_tp_pips > Decimal::ZERO {
                (max_fav_pips / target_tp_pips) * dec!(100.0)
            } else {
                Decimal::ZERO
            };

            diagnostics.push(LossDiagnostics {
                order: (*loss).clone(),
                setup_candles,
                trade_candles: trade_candles.clone(),
                max_favorable_pips: max_fav_pips,
                target_tp_pips,
                risk_sl_pips,
                mfe_pct_of_tp,
                duration_bars: trade_candles.len(),
            });
        }

        // Statistical Categories
        let total_loss_count = diagnostics.len();
        let near_tp_losses = diagnostics
            .iter()
            .filter(|d| d.mfe_pct_of_tp >= dec!(50.0))
            .count();
        let instant_fakeouts = diagnostics
            .iter()
            .filter(|d| d.duration_bars <= 3 && d.mfe_pct_of_tp < dec!(25.0))
            .count();
        let slow_churn = diagnostics.iter().filter(|d| d.duration_bars > 12).count();

        println!(
            "📊 ROOT-CAUSE BREAKDOWN OF LOSSES ({} Trades):",
            total_loss_count
        );
        println!(
            "  1. 🎯 Near-TP Reversals (MFE >= 50% TP, lalu berbalik ke SL) : {} ({:.1}%)",
            near_tp_losses,
            (Decimal::from(near_tp_losses) / Decimal::from(total_loss_count)) * dec!(100.0)
        );
        println!(
            "  2. ⚡ Immediate Fakeouts / Whipsaws (SL tembus dlm <= 3 bar)   : {} ({:.1}%)",
            instant_fakeouts,
            (Decimal::from(instant_fakeouts) / Decimal::from(total_loss_count)) * dec!(100.0)
        );
        println!(
            "  3. ⏳ Slow Stagnation Churn (Floating lambat > 12 bar)        : {} ({:.1}%)",
            slow_churn,
            (Decimal::from(slow_churn) / Decimal::from(total_loss_count)) * dec!(100.0)
        );

        // Print 3 Detailed Case Studies per pair
        println!("\n🔍 SAMPLE CANDLE-LEVEL FORENSIC CASE STUDIES (Last 3 Losses):");
        for (i, diag) in diagnostics.iter().rev().take(3).enumerate() {
            let o = &diag.order;
            let action_str = match o.action {
                SignalAction::BuyLimit => "BUY LIMIT",
                SignalAction::SellLimit => "SELL LIMIT",
                _ => "OTHER",
            };

            println!("  ─────────────────────────────────────────────────────────────────────────");
            let close_str = match o.close_time {
                Some(t) => t.format("%Y-%m-%d %H:%M UTC").to_string(),
                None => "N/A".to_string(),
            };
            println!(
                "  [Case #{}] {} | Action: {} | Fill: {} | Exit: {}",
                i + 1,
                pair_str,
                action_str,
                o.open_time.format("%Y-%m-%d %H:%M UTC"),
                close_str
            );
            println!(
                "  Entry: {} | SL: {} ({:.1} pips) | TP: {} ({:.1} pips)",
                o.open_price, o.stop_loss, diag.risk_sl_pips, o.take_profit, diag.target_tp_pips
            );
            println!(
                "  Peak MFE: +{:.1} pips ({:.1}% of TP target reached before SL hit)",
                diag.max_favorable_pips, diag.mfe_pct_of_tp
            );
            println!("  Candles Held: {} bars", diag.duration_bars);

            println!("  Candle Progression During Trade:");
            for (bar_idx, c) in diag.trade_candles.iter().enumerate().take(8) {
                let candle_type = if c.close >= c.open {
                    "🟩 BULL"
                } else {
                    "🟥 BEAR"
                };
                let diff_from_entry = match o.action {
                    SignalAction::BuyLimit => spec.price_diff_to_pips(c.close - o.open_price),
                    _ => spec.price_diff_to_pips(o.open_price - c.close),
                };
                let pnl_sign = if diff_from_entry >= Decimal::ZERO {
                    "+"
                } else {
                    ""
                };
                println!(
                    "    Bar {:>2} [{}] {} O:{:.5} H:{:.5} L:{:.5} C:{:.5} | Close PnL: {}{:.1} pips",
                    bar_idx + 1,
                    c.timestamp.format("%H:%M"),
                    candle_type,
                    c.open,
                    c.high,
                    c.low,
                    c.close,
                    pnl_sign,
                    diff_from_entry
                );
            }
        }
        println!("\n");
    }
}
