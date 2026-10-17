use chrono::Utc;
use domain::models::{AppConfig, PolaNStrategy, RiskProfile, Symbol};
use domain::ports::StrategyPort;
use domain::strategies::{EnsembleStrategy, SmcLiquiditySweepStrategy};
use std::collections::HashMap;
use std::sync::Arc;

mod commands;
mod feed;

use commands::{
    run_compliance_and_publish, run_eda, run_pair_breakdown, run_tournament, run_walk_forward,
};
use feed::{load_real_market_candles, HistoricalMarketFeed};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔═════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║       📈 TRADERS FAMILY QUANT ANALYST CLI & STRATEGY TOURNAMENT (2026)                  ║");
    println!("╚═════════════════════════════════════════════════════════════════════════════════════════╝");

    // 1. Load Konfigurasi Terpusat (config.toml)
    let config = AppConfig::load_from_file_or_default("config.toml");
    println!(
        "Memuat Konfigurasi: config.toml (Env: {})",
        config.environment
    );
    println!("Target Monetisasi: Kualifikasi Analis (>= 300 VP & Scorecard 7-Faktor Max)\n");

    let symbols: Vec<Symbol> = config
        .active_symbols
        .iter()
        .filter_map(|s| Symbol::from_symbol_str(s))
        .collect();

    // 2. Siapkan Real Market Feed Multi-Pair (100% Data Nyata)
    println!("📡 Memuat data historis pasar nyata dari data/historical/...");
    let mut feed_map = HashMap::new();
    for symbol in &symbols {
        let candles = load_real_market_candles(symbol)?;
        println!(
            "  ✅ {}: {} real H1 candles termuat.",
            symbol.to_compact_string(),
            candles.len()
        );
        feed_map.insert(symbol.to_compact_string(), candles);
    }
    println!();

    let shared_feed = Arc::new(HistoricalMarketFeed {
        candles: feed_map.clone(),
        spread_pips: config.backtest.simulation_spread_pips,
    });

    let risk_profile = RiskProfile::from_config(&config.risk_management);

    // 3. EXPLORATORY DATA ANALYSIS (EDA) & AUDIT KESEHATAN DATASET
    run_eda(&symbols, &feed_map);

    // 4. Daftarkan Strategi untuk Turnamen Kuantitatif
    let pola_n_strategy: Arc<dyn StrategyPort> =
        Arc::new(PolaNStrategy::from_config(&config.strategy_pola_n));
    let smc_strategy: Arc<dyn StrategyPort> = Arc::new(SmcLiquiditySweepStrategy::default());
    let ensemble_strategy: Arc<dyn StrategyPort> = Arc::new(EnsembleStrategy::new(
        "Ensemble-PolaN-SMC-Hybrid",
        pola_n_strategy.clone(),
        smc_strategy.clone(),
    ));

    let tournament_strategies: Vec<Arc<dyn StrategyPort>> = vec![
        pola_n_strategy.clone(),
        smc_strategy.clone(),
        ensemble_strategy.clone(),
    ];

    let sim_start = feed_map
        .values()
        .next()
        .and_then(|v| v.first())
        .map(|c| c.timestamp)
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(10_000));
    let sim_end = feed_map
        .values()
        .next()
        .and_then(|v| v.last())
        .map(|c| c.timestamp)
        .unwrap_or_else(Utc::now);

    // 5. Jalankan Strategy Tournament Arena
    run_tournament(
        shared_feed.clone(),
        &risk_profile,
        &config.backtest,
        &tournament_strategies,
        &symbols,
        sim_start,
        sim_end,
    )
    .await?;

    // 6. Rincian Performa Juara 1 (Pola N Strategy) Per-Pair
    run_pair_breakdown(
        shared_feed.clone(),
        pola_n_strategy.clone(),
        &risk_profile,
        &config.backtest,
        &symbols,
        sim_start,
        sim_end,
    )
    .await?;

    // 7. Komparasi Head-to-Head: Backtest (In-Sample 70%) vs Forward Test (Out-of-Sample 30%)
    run_walk_forward(
        &feed_map,
        pola_n_strategy.clone(),
        &risk_profile,
        &config.backtest,
        &symbols,
        sim_start,
        sim_end,
    )
    .await?;

    // 8. Test Validasi Kepatuhan TF Compliance Guard & Publishing
    run_compliance_and_publish(&config).await?;

    Ok(())
}
