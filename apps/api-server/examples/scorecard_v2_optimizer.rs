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
    let storage: Arc<dyn StoragePort> = Arc::new(InMemoryStorage);

    println!("\n🔍 BENCHMARK KOMPARATIF: V1 Baseline vs V2 Adaptive Next-Gen");

    let configs: Vec<(&str, Arc<dyn StrategyPort>)> = vec![
        (
            "V1 Baseline (Production)",
            Arc::new(PolaNStrategy::v1_production()),
        ),
        (
            "V2 Adaptive (Next-Gen)",
            Arc::new(PolaNStrategy::v2_adaptive()),
        ),
    ];

    for (label, strat) in configs {
        let audit_service = QuantAuditService::new(market_adapter.clone(), strat, storage.clone());
        let full_audit = audit_service.get_full_audit().await.expect("Audit failed");
        let sc = full_audit.scorecard;

        println!(
            "\n════════════════════════════════════════════════════════════════════════════════"
        );
        println!("🚀 MODEL: {}", label);
        println!(
            "════════════════════════════════════════════════════════════════════════════════"
        );
        println!(
            "Total Score: {} / {} ({}%) | Tier: {} | Rev Share: {}%",
            sc.total_score,
            sc.max_score,
            sc.score_pct,
            sc.revenue_share_tier,
            sc.max_revenue_share_pct
        );
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        for p in &sc.pillars {
            println!(
                "  [{:<2}] {:<24} | {:>5}% wt | Nilai: {:<20} | Skor: {}/4 ({})",
                p.code, p.name, p.weight_pct, p.our_value, p.achieved_points, p.status
            );
        }
    }
}
