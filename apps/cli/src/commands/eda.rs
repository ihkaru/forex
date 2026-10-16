use application::services::EdaService;
use domain::models::{Candle, Symbol};
use std::collections::HashMap;

pub fn run_eda(symbols: &[Symbol], feed_map: &HashMap<String, Vec<Candle>>) {
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("🔍 1. EXPLORATORY DATA ANALYSIS (EDA) & AUDIT KESEHATAN DATASET");
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!(
        "{:<8} {:<8} {:<9} {:<10} {:<10} {:<13} {:<11} {:<24}",
        "PAIR",
        "CANDLES",
        "DURASI",
        "MIN PRICE",
        "MAX PRICE",
        "AVG RANGE(p)",
        "MAX BAR(p)",
        "HEALTH SCORECARD"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    for symbol in symbols {
        if let Some(candles) = feed_map.get(&symbol.to_compact_string()) {
            let eda = EdaService::analyze(symbol, candles);
            let dur_str = format!("{:.1} hari", eda.total_duration_days);
            println!(
                "{:<8} {:<8} {:<9} {:<10.5} {:<10.5} {:<13.1} {:<11.1} {:<24}",
                symbol.to_compact_string(),
                eda.total_candles,
                dur_str,
                eda.min_price,
                eda.max_price,
                eda.avg_bar_range_pips,
                eda.max_single_bar_pips,
                eda.health_status
            );
        }
    }
    println!("═════════════════════════════════════════════════════════════════════════════════════════\n");
}
