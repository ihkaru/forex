use application::services::StrategyBenchmarkService;
use chrono::{DateTime, Utc};
use domain::models::{BacktestConfig, RiskProfile, Symbol, Timeframe};
use domain::ports::{MarketDataPort, StrategyPort};
use std::sync::Arc;

pub async fn run_tournament(
    shared_feed: Arc<dyn MarketDataPort>,
    risk_profile: &RiskProfile,
    backtest_config: &BacktestConfig,
    tournament_strategies: &[Arc<dyn StrategyPort>],
    symbols: &[Symbol],
    sim_start: DateTime<Utc>,
    sim_end: DateTime<Utc>,
) -> anyhow::Result<()> {
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("🏆 2. STRATEGY TOURNAMENT LEADERBOARD (KOMPARASI MULTI-STRATEGI KUANTITATIF)");
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );

    let benchmark_service = StrategyBenchmarkService::new(
        shared_feed,
        risk_profile.clone(),
        backtest_config.clone(),
    );

    let leaderboard = benchmark_service
        .run_tournament(
            tournament_strategies,
            symbols,
            Timeframe::H1,
            sim_start,
            sim_end,
        )
        .await?;

    println!(
        "{:<4} {:<28} {:<8} {:<8} {:<10} {:<12} {:<8} {:<8} {:<18}",
        "RANK",
        "STRATEGY NAME",
        "TRADES",
        "WIN(%)",
        "RAW PIPS",
        "VALUED PIPS",
        "PF",
        "REC.F",
        "TF STATUS"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    for entry in &leaderboard {
        let medal = match entry.rank {
            1 => "🥇",
            2 => "🥈",
            3 => "🥉",
            _ => "  ",
        };

        println!(
            "{} {:<2} {:<28} {:<8} {:<8.1} {:<10.1} {:<12.1} {:<8.2} {:<8.2} {:<18}",
            medal,
            entry.rank,
            entry.strategy_name,
            entry.total_trades,
            entry.win_rate_percent,
            entry.total_raw_pips,
            entry.total_valued_pips,
            entry.profit_factor,
            entry.recovery_factor,
            entry.tf_status
        );
    }
    println!("═════════════════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
