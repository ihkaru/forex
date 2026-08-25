use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::DomainError;
use crate::ports::audit::ScorecardAuditReport;
use crate::ports::StrategyPort;

/// Hasil verifikasi status invariant mutlak Traders Family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantCheckResult {
    pub rule_id: u8,
    pub name: String,
    pub description: String,
    pub total_evaluated: usize,
    pub violations_count: usize,
    pub is_passed: bool,
    pub details: String,
}

/// Rincian evaluasi kelulusan per bulan kalender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyQualificationItem {
    pub year: i32,
    pub month: u32,
    pub settled_trades: usize,
    pub win_trades: usize,
    pub loss_trades: usize,
    pub win_rate_pct: Decimal,
    pub raw_pips: Decimal,
    pub valued_pips: Decimal,
    pub is_vp_qualified: bool,     // VP >= 300
    pub is_volume_qualified: bool, // Settled >= 5
    pub is_month_qualified: bool,  // VP >= 300 && Settled >= 5
    pub tf_points_earned: Decimal, // Calculated via tiered profit formula
    pub profit_factor: Decimal,
}

/// Rincian perhitungan tiering reward Valued Pips per bulan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipsTieringAuditResult {
    pub total_qualified_months: usize,
    pub total_disqualified_months: usize,
    pub total_tf_points: Decimal,
    pub total_cashout_idr: Decimal,
    pub base_tier2_points: Decimal, // Tier 2: 300..=500 VP (100% multiplier)
    pub surplus_tier3_points: Decimal, // Tier 3: >500 VP (20% diminishing bonus)
    pub current_medals: usize,
    pub current_level_name: String,
    pub current_multiplier: Decimal,
}

/// Laporan Audit Komprehensif Deterministik Kualifikasi Traders Family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfQualificationAuditReport {
    pub strategy_name: String,
    pub audit_timestamp: DateTime<Utc>,
    pub from_date: DateTime<Utc>,
    pub to_date: DateTime<Utc>,
    pub total_analyzed_bars: usize,
    pub total_trades: usize,
    pub total_valued_pips: Decimal,
    pub overall_win_rate_pct: Decimal,
    pub overall_profit_factor: Decimal,
    pub max_drawdown_pct: Decimal,
    pub recovery_factor: Decimal,
    pub hard_invariants: Vec<InvariantCheckResult>,
    pub all_invariants_passed: bool,
    pub monthly_qualification: Vec<MonthlyQualificationItem>,
    pub qualification_pass_rate_pct: Decimal,
    pub pips_tiering: PipsTieringAuditResult,
    pub scorecard_7pillars: ScorecardAuditReport,
    pub is_fully_tf_qualified: bool,
}

/// Port Trait: Audit Kualifikasi Deterministik Independen Lintas-Strategi
#[async_trait]
pub trait TfQualificationAuditPort: Send + Sync {
    /// Menjalankan audit kepatuhan dan kualifikasi lengkap terhadap suatu strategi
    async fn audit_strategy(
        &self,
        strategy: Arc<dyn StrategyPort>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<TfQualificationAuditReport, DomainError>;
}
