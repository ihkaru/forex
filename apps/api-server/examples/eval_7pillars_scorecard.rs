use api_server::state::RealHistoricalMarketAdapter;
use application::services::audit::QuantAuditService;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, MarketDataSource, Order, PolaNStrategy, Signal, Symbol, Timeframe};
use domain::ports::audit::QuantAuditPort;
use domain::ports::storage::StoragePort;
use domain::ports::StrategyPort;
use std::sync::Arc;
use uuid::Uuid;

struct InMemoryStorage;

#[async_trait::async_trait]
impl StoragePort for InMemoryStorage {
    async fn save_signal(&self, _s: &Signal) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_signal(&self, _id: Uuid) -> Result<Option<Signal>, DomainError> {
        Ok(None)
    }
    async fn get_active_signals(&self) -> Result<Vec<Signal>, DomainError> {
        Ok(vec![])
    }
    async fn save_candles(&self, _candles: &[Candle]) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_candles(
        &self,
        _sym: &Symbol,
        _tf: Timeframe,
        _l: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        Ok(vec![])
    }
    async fn get_high_watermark(
        &self,
        _sym: &Symbol,
        _tf: Timeframe,
        _s: MarketDataSource,
    ) -> Result<Option<DateTime<Utc>>, DomainError> {
        Ok(None)
    }
    async fn save_order(&self, _o: &Order) -> Result<(), DomainError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let market_adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let strategy: Arc<dyn StrategyPort> = Arc::new(PolaNStrategy::v2_adaptive());
    let storage: Arc<dyn StoragePort> = Arc::new(InMemoryStorage);

    let audit_service = QuantAuditService::new(market_adapter, strategy, storage);

    println!("\n🔍 Menjalankan Audit Kuantitatif Portofolio Multi-Pair (10-Tahun)...");
    let full_audit = audit_service.get_full_audit().await.expect("Audit failed");
    let sc = full_audit.scorecard;

    println!("\n╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    TRADERS FAMILY MONETIZATION ENGINE & 7-PILLAR SCORECARD                               ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!(
        "║  TOTAL SCORE     : {:>2} / {:>2} ({:>5}%) │  TIER STATUS    : {:<20}                     ║",
        sc.total_score,
        sc.max_score,
        sc.score_pct,
        sc.revenue_share_tier
    );
    println!(
        "║  PARTNERSHIP     : Priority Official  │  REV SHARING    : {:>2}% REVENUE SHARE (MAX)               ║",
        sc.max_revenue_share_pct
    );
    println!("╠══════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Kode │ Pilar Penilaian          │ Bobot  │ Nilai Riil Portofolio    │ Target Benchmark  │ Skor │ Status  ║");
    println!("╟──────┼──────────────────────────┼────────┼──────────────────────────┼───────────────────┼──────┼─────────╢");

    for p in &sc.pillars {
        println!(
            "║ {:<4} │ {:<24} │ {:>5}% │ {:<24} │ {:<17} │ {:>1}/4  │ {:<7} ║",
            p.code,
            p.name,
            p.weight_pct,
            p.our_value,
            p.benchmark_rule,
            p.achieved_points,
            p.status
        );
    }
    println!("╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝\n");
}
