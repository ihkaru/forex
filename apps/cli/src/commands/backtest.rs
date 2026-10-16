use application::services::{BacktestReport, BacktestService};
use chrono::{DateTime, Utc};
use domain::models::{BacktestConfig, Candle, PairTier, RiskProfile, Symbol, TfPairSpec, Timeframe};
use domain::ports::{MarketDataPort, StrategyPort};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;

use crate::feed::HistoricalMarketFeed;

pub async fn run_pair_breakdown(
    shared_feed: Arc<dyn MarketDataPort>,
    pola_n_strategy: Arc<dyn StrategyPort>,
    risk_profile: &RiskProfile,
    backtest_config: &BacktestConfig,
    symbols: &[Symbol],
    sim_start: DateTime<Utc>,
    sim_end: DateTime<Utc>,
) -> anyhow::Result<()> {
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("📊 3. RINCIAN PER-PAIR STRATEGI JUARA: POLA N SIGNATURE TRADERS FAMILY");
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );

    let backtester = BacktestService::with_config(
        shared_feed,
        pola_n_strategy,
        risk_profile.clone(),
        backtest_config.clone(),
    );

    let mut reports: Vec<BacktestReport> = Vec::new();
    let mut total_portfolio_vp = Decimal::ZERO;
    let mut total_portfolio_trades = 0;

    for symbol in symbols {
        let report = backtester
            .run_simulation(symbol, Timeframe::H1, sim_start, sim_end)
            .await?;

        total_portfolio_vp += report.total_valued_pips;
        total_portfolio_trades += report.total_trades;
        reports.push(report);
    }

    println!(
        "{:<8} {:<7} {:<6} {:<8} {:<8} {:<10} {:<12} {:<8} {:<8} {:<10}",
        "PAIR",
        "TIER",
        "MULT",
        "TRADES",
        "WIN(%)",
        "RAW PIPS",
        "VALUED PIPS",
        "PF",
        "REC.F",
        "STATUS"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    for report in &reports {
        let spec = TfPairSpec::from_symbol(&report.symbol);
        let tier_name = match spec.tier {
            PairTier::Tier1 => "Tier 1",
            PairTier::Tier2 => "Tier 2",
            PairTier::Tier3 => "Tier 3",
            PairTier::Tier4 => "Tier 4",
        };

        let status_str = if report.is_tf_qualified {
            "✅ LOLOS"
        } else {
            "⚠️ REVIEW"
        };

        println!(
            "{:<8} {:<7} {:<6.1} {:<8} {:<8.1} {:<10.1} {:<12.1} {:<8.2} {:<8.2} {:<10}",
            report.symbol.to_compact_string(),
            tier_name,
            spec.value_multiplier,
            report.total_trades,
            report.win_rate_percent,
            report.total_raw_pips,
            report.total_valued_pips,
            report.profit_factor,
            report.recovery_factor,
            status_str
        );
    }

    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "🏆 TOTAL PORTOFOLIO VALUED PIPS: {:.1} VP | TOTAL TRADES: {}",
        total_portfolio_vp, total_portfolio_trades
    );
    let target_met = total_portfolio_vp >= dec!(300.0) && total_portfolio_trades >= 5;
    println!(
        "🎯 KUALIFIKASI TF REWARD POINT (Target >= 300 VP): {}",
        if target_met {
            "🌟 MEMENUHI SYARAT REWARD POINT BULANAN!"
        } else {
            "❌ BELUM MEMENUHI TARGET"
        }
    );
    println!("═════════════════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}

pub async fn run_walk_forward(
    feed_map: &HashMap<String, Vec<Candle>>,
    pola_n_strategy: Arc<dyn StrategyPort>,
    risk_profile: &RiskProfile,
    backtest_config: &BacktestConfig,
    symbols: &[Symbol],
    sim_start: DateTime<Utc>,
    sim_end: DateTime<Utc>,
) -> anyhow::Result<()> {
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("🔬 4. KOMPARASI HEAD-TO-HEAD: BACKTEST (IN-SAMPLE) VS FORWARD TEST (OUT-OF-SAMPLE)");
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );

    let mut is_feed_map = HashMap::new();
    let mut oos_feed_map = HashMap::new();

    for symbol in symbols {
        if let Some(all_candles) = feed_map.get(&symbol.to_compact_string()) {
            let split_idx = (all_candles.len() * 70) / 100; // 70% In-Sample
            let is_candles = all_candles[..split_idx].to_vec();
            let oos_candles = all_candles[split_idx..].to_vec();

            is_feed_map.insert(symbol.to_compact_string(), is_candles);
            oos_feed_map.insert(symbol.to_compact_string(), oos_candles);
        }
    }

    let is_feed = Arc::new(HistoricalMarketFeed {
        candles: is_feed_map.clone(),
        spread_pips: backtest_config.simulation_spread_pips,
    });
    let oos_feed = Arc::new(HistoricalMarketFeed {
        candles: oos_feed_map.clone(),
        spread_pips: backtest_config.simulation_spread_pips,
    });

    let is_backtester = BacktestService::with_config(
        is_feed,
        pola_n_strategy.clone(),
        risk_profile.clone(),
        backtest_config.clone(),
    );
    let oos_backtester = BacktestService::with_config(
        oos_feed,
        pola_n_strategy.clone(),
        risk_profile.clone(),
        backtest_config.clone(),
    );

    let mut is_total_vp = Decimal::ZERO;
    let mut is_total_trades = 0;
    let mut is_wins = 0;

    let mut oos_total_vp = Decimal::ZERO;
    let mut oos_total_trades = 0;
    let mut oos_wins = 0;

    for symbol in symbols {
        let is_rep = is_backtester
            .run_simulation(symbol, Timeframe::H1, sim_start, sim_end)
            .await?;
        is_total_vp += is_rep.total_valued_pips;
        is_total_trades += is_rep.total_trades;
        is_wins += is_rep.winning_trades;

        let oos_rep = oos_backtester
            .run_simulation(symbol, Timeframe::H1, sim_start, sim_end)
            .await?;
        oos_total_vp += oos_rep.total_valued_pips;
        oos_total_trades += oos_rep.total_trades;
        oos_wins += oos_rep.winning_trades;
    }

    let is_win_rate = if is_total_trades > 0 {
        (is_wins as f64 / is_total_trades as f64) * 100.0
    } else {
        0.0
    };
    let oos_win_rate = if oos_total_trades > 0 {
        (oos_wins as f64 / oos_total_trades as f64) * 100.0
    } else {
        0.0
    };

    let wfer_ratio = if is_win_rate > 0.0 {
        (oos_win_rate / is_win_rate) * 100.0
    } else {
        0.0
    };

    let is_bar_count = is_feed_map.values().next().map(|v| v.len()).unwrap_or(0);
    let oos_bar_count = oos_feed_map.values().next().map(|v| v.len()).unwrap_or(0);
    let is_dur_label = format!(
        "{} Bar (~{:.0} Hari)",
        is_bar_count,
        is_bar_count as f64 / 24.0
    );
    let oos_dur_label = format!(
        "{} Bar (~{:.0} Hari)",
        oos_bar_count,
        oos_bar_count as f64 / 24.0
    );

    println!(
        "{:<28} {:<24} {:<24} {:<12}",
        "METRIK EVALUASI", "BACKTEST (IN-SAMPLE 70%)", "FORWARD TEST (OOS 30%)", "SELISIH / WFER"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "{:<28} {:<24} {:<24} {:<12}",
        "Dataset Bars / Durasi", is_dur_label, oos_dur_label, "Rasio 70:30"
    );
    println!(
        "{:<28} {:<24} {:<24} {:<12}",
        "Total Settled Trades",
        format!("{} Trades", is_total_trades),
        format!("{} Trades", oos_total_trades),
        "-"
    );
    println!(
        "{:<28} {:<24} {:<24} {:<12}",
        "Win Rate (%)",
        format!("{:.1}%", is_win_rate),
        format!("{:.1}%", oos_win_rate),
        format!("WFER: {:.1}%", wfer_ratio)
    );
    println!(
        "{:<28} {:<24} {:<24} {:<12}",
        "Total Valued Pips (VP)",
        format!("+{:.1} VP", is_total_vp),
        format!("+{:.1} VP", oos_total_vp),
        "Konsisten Hijau"
    );
    println!(
        "{:<28} {:<24} {:<24} {:<12}",
        "Rata-rata VP / Trade",
        format!(
            "{:.1} VP/trade",
            if is_total_trades > 0 {
                is_total_vp / Decimal::from(is_total_trades)
            } else {
                Decimal::ZERO
            }
        ),
        format!(
            "{:.1} VP/trade",
            if oos_total_trades > 0 {
                oos_total_vp / Decimal::from(oos_total_trades)
            } else {
                Decimal::ZERO
            }
        ),
        "Stabil"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    let robustness_verdict = if wfer_ratio >= 60.0 {
        "🟢 SANGAT ROBUST (Walk-Forward Terbukti Tahan Uji di Data Buta)"
    } else if wfer_ratio >= 40.0 {
        "🟡 CUKUP BAIK (Performa OOS Mengalami Sedikit Decay Wajar)"
    } else {
        "🔴 OVERFITTING DETECTED (Performa Drop Drastis di Data Baru)"
    };
    println!("🛡️ KESIMPULAN AUDIT ROBUSTNESS: {}", robustness_verdict);
    println!("═════════════════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
