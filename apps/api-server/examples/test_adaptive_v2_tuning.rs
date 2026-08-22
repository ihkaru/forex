use api_server::state::RealHistoricalMarketAdapter;
use application::services::audit::QuantAuditService;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, MarketDataSource, Order, PolaNStrategy, Signal, Symbol, Timeframe};
use domain::ports::audit::QuantAuditPort;
use domain::ports::storage::StoragePort;
use rust_decimal_macros::dec;
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

    println!("\n🔍 PARAMETER SWEEP: MENGEJAR SCORECARD 24 - 28 POIN (LEGEND TIER)");
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    let configs = [
        ("V2-Swing(4,2)-RR:1.20", 4, 2, dec!(1.20)),
        ("V2-Swing(4,2)-RR:1.25", 4, 2, dec!(1.25)),
        ("V2-Swing(4,3)-RR:1.20", 4, 3, dec!(1.20)),
        ("V2-Swing(4,3)-RR:1.25", 4, 3, dec!(1.25)),
        ("V2-Swing(5,3)-RR:1.20", 5, 3, dec!(1.20)),
    ];

    for (label, left, right, rr) in configs {
        let strat = Arc::new(PolaNStrategy::with_params(
            label,
            left,
            right,
            dec!(0.00020),
            rr,
        ));
        let audit_service = QuantAuditService::new(market_adapter.clone(), strat, storage.clone());
        if let Ok(full_audit) = audit_service.get_full_audit().await {
            let sc = full_audit.scorecard;
            let rf = sc
                .pillars
                .iter()
                .find(|p| p.code == "RF")
                .map(|p| p.our_value.as_str())
                .unwrap_or("0");
            let pf = sc
                .pillars
                .iter()
                .find(|p| p.code == "PF")
                .map(|p| p.our_value.as_str())
                .unwrap_or("0");
            let sb = sc
                .pillars
                .iter()
                .find(|p| p.code == "SB")
                .map(|p| p.our_value.as_str())
                .unwrap_or("0");
            println!(
                "{:<24} ➔ Total: {:>2}/28 ({:>4.1}%) | Tier: {:<15} | RF: {:>5} | PF: {:>4} | Vol: {:>7}",
                label,
                sc.total_score,
                sc.score_pct,
                sc.revenue_share_tier,
                rf,
                pf,
                sb
            );
        }
    }
}
