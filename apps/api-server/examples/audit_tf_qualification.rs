#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    clippy::all
)]
use api_server::state::RealHistoricalMarketAdapter;
use application::services::TfQualificationService;
use chrono::{TimeZone, Utc};
use domain::models::{PolaNStrategy, RiskProfile};
use domain::ports::TfQualificationAuditPort;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let qualification_service = TfQualificationService::new(adapter, RiskProfile::default());

    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let strategies = vec![
        (
            "TF Pola N Institutional (v3 Gold Specialist Pro)",
            Arc::new(PolaNStrategy::v3_gold_pro()) as Arc<dyn domain::ports::StrategyPort>,
        ),
        (
            "TF Pola N Adaptive (v2 Gold Specialist)",
            Arc::new(PolaNStrategy::v2_adaptive()) as Arc<dyn domain::ports::StrategyPort>,
        ),
        (
            "TF Pola N Production (v1 Baseline)",
            Arc::new(PolaNStrategy::v1_production()) as Arc<dyn domain::ports::StrategyPort>,
        ),
    ];

    println!("\n==========================================================================================================");
    println!("     TRADERS FAMILY DETERMINISTIC QUALIFICATION & COMPLIANCE AUDIT ENGINE (10-YEAR HISTORICAL)           ");
    println!("==========================================================================================================");

    for (strat_name, strat) in strategies {
        println!("\n🔍 AUDITING STRATEGY: {strat_name}");
        println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────");

        match qualification_service.audit_strategy(strat, from, to).await {
            Ok(report) => {
                // 1. Executive Performance Summary
                println!("📊 1. RINGKASAN PERFORMA KUANTITATIF (2015 - 2026):");
                println!(
                    "   • Total Bar Dianalisis  : {} Bar H1",
                    report.total_analyzed_bars
                );
                println!(
                    "   • Total Sinyal Settled  : {} Trades",
                    report.total_trades
                );
                println!(
                    "   • Total Valued Pips     : +{:.1} VP",
                    report.total_valued_pips
                );
                println!(
                    "   • Win Rate              : {:.1}%",
                    report.overall_win_rate_pct
                );
                println!(
                    "   • Profit Factor         : {:.2}",
                    report.overall_profit_factor
                );
                println!(
                    "   • Max Drawdown          : {:.1}% (Kelly Controlled)",
                    report.max_drawdown_pct
                );
                println!("   • Recovery Factor       : {:.1}", report.recovery_factor);

                // 2. 8 Hard Invariants Verification Table
                println!(
                    "\n🛡️ 2. AUDIT 8 INVARIANT MUTLAK TRADERS FAMILY (ZERO-VIOLATION GUARANTEE):"
                );
                println!("   ┌─────┬───────────────────────────────────────────────┬────────────┬────────────┬─────────┐");
                println!("   │ No  │ Aturan Invariant                              │ Evaluasi   │ Pelanggaran│ Status  │");
                println!("   ├─────┼───────────────────────────────────────────────┼────────────┼────────────┼─────────┤");
                for inv in &report.hard_invariants {
                    let status_badge = if inv.is_passed {
                        "✅ PASS"
                    } else {
                        "❌ FAIL"
                    };
                    println!(
                        "   │ #{:<2} │ {:<45} │ {:>10} │ {:>10} │ {:<7} │",
                        inv.rule_id,
                        inv.name,
                        inv.total_evaluated,
                        inv.violations_count,
                        status_badge
                    );
                }
                println!("   └─────┴───────────────────────────────────────────────┴────────────┴────────────┴─────────┘");

                // 3. Monthly Qualification & Tiered Pips Reward
                println!("\n📅 3. EVALUASI KUALIFIKASI BULANAN & SISTEM TIERING PIPS (MEDAL PROGRESSION):");
                println!(
                    "   • Bulan Lolos Kualifikasi   : {} dari {} bulan ({:.1}%)",
                    report.pips_tiering.total_qualified_months,
                    report.monthly_qualification.len(),
                    report.qualification_pass_rate_pct
                );
                println!(
                    "   • Akumulasi TF Medal        : {} Medals",
                    report.pips_tiering.current_medals
                );
                println!(
                    "   • Level Analis Dicapai      : {} (Multiplier {:.1}x)",
                    report.pips_tiering.current_level_name, report.pips_tiering.current_multiplier
                );
                println!(
                    "   • Poin Tier 2 (300-500 VP)  : +{:.1} TF Point",
                    report.pips_tiering.base_tier2_points
                );
                println!(
                    "   • Poin Tier 3 (>500 VP Surp): +{:.1} TF Point",
                    report.pips_tiering.surplus_tier3_points
                );
                println!(
                    "   • TOTAL TF POINT BULANAN    : +{:.1} TF Point",
                    report.pips_tiering.total_tf_points
                );
                println!(
                    "   • POTENSI REWARD CASHOUT    : Rp {}",
                    report.pips_tiering.total_cashout_idr
                );

                // Sample recent months
                println!("\n   [Rincian 6 Bulan Terakhir]:");
                println!("   ┌──────────┬──────────┬──────────┬────────────┬────────────┬──────────┬─────────────┐");
                println!("   │ Periode  │ Settled  │ Win Rate │ Valued Pips│ VP >= 300  │ Trd >= 5 │ Status      │");
                println!("   ├──────────┼──────────┼──────────┼────────────┼────────────┼──────────┼─────────────┤");
                for mo in report.monthly_qualification.iter().rev().take(6) {
                    let vp_badge = if mo.is_vp_qualified {
                        "✅ PASS"
                    } else {
                        "❌ FAIL"
                    };
                    let vol_badge = if mo.is_volume_qualified {
                        "✅ PASS"
                    } else {
                        "❌ FAIL"
                    };
                    let mo_badge = if mo.is_month_qualified {
                        "🏆 LOLOS"
                    } else {
                        "❌ GAGAL"
                    };
                    println!(
                        "   │ {:04}-{:02}  │ {:>8} │ {:>7.1}% │ {:>9.1} VP│ {:<10} │ {:<8} │ {:<11} │",
                        mo.year, mo.month, mo.settled_trades, mo.win_rate_pct, mo.valued_pips, vp_badge, vol_badge, mo_badge
                    );
                }
                println!("   └──────────┴──────────┴──────────┴────────────┴────────────┴──────────┴─────────────┘");

                // 4. 7-Pillars Final Scorecard
                println!("\n🏆 4. SCORECARD 7-PILAR & STATUS KEMITRAAN PRIORITY CHANNEL:");
                println!(
                    "   • Total Skor Scorecard : {} / 28 ({:.1}%)",
                    report.scorecard_7pillars.total_score, report.scorecard_7pillars.score_pct
                );
                println!(
                    "   • Status Revenue Share : {} (Maksimal {}% Revenue Sharing Subscriber)",
                    report.scorecard_7pillars.revenue_share_tier,
                    report.scorecard_7pillars.max_revenue_share_pct
                );

                if report.is_fully_tf_qualified {
                    println!("\n✨ KESIMPULAN AUDIT: STRATEGI TERBUKTI 100% MEMENUHI SELURUH KRITERIA TRADERS FAMILY!");
                } else {
                    println!("\n⚠️ KESIMPULAN AUDIT: Strategi belum memenuhi kriteria kualifikasi penuh.");
                }
            }
            Err(e) => {
                eprintln!("❌ Error saat menjalankan audit: {e}");
            }
        }
        println!("==========================================================================================================");
    }
}
