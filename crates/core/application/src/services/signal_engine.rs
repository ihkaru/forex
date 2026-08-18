use std::sync::Arc;
use tracing::{error, info, warn};

use domain::errors::DomainError;
use domain::models::{RiskProfile, Signal, Symbol, TfComplianceGuard, Timeframe};
use domain::ports::{MarketContext, MarketDataPort, SignalPublisherPort, StoragePort, StrategyPort};

/// `SignalEngineService` mengorkestrasi pipeline pemindaian pasar,
/// evaluasi strategi modular, validasi regulasi ketat Traders Family (Zero-Penalty),
/// dan broadcast sinyal ke publisher (e.g. Trader Family).
pub struct SignalEngineService {
    market_data: Arc<dyn MarketDataPort>,
    publishers: Vec<Arc<dyn SignalPublisherPort>>,
    storage: Arc<dyn StoragePort>,
    strategies: Vec<Arc<dyn StrategyPort>>,
    risk_profile: RiskProfile,
}

impl SignalEngineService {
    pub fn new(
        market_data: Arc<dyn MarketDataPort>,
        publishers: Vec<Arc<dyn SignalPublisherPort>>,
        storage: Arc<dyn StoragePort>,
        strategies: Vec<Arc<dyn StrategyPort>>,
        risk_profile: RiskProfile,
    ) -> Self {
        Self {
            market_data,
            publishers,
            storage,
            strategies,
            risk_profile,
        }
    }

    /// Evaluasi satu pasang mata uang (symbol) dan timeframe untuk peluang sinyal baru
    pub async fn process_symbol(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
    ) -> Result<Option<Signal>, DomainError> {
        info!("Mengevaluasi pasar untuk {} ({:?})", symbol, timeframe);

        // 1. Tarik data pasar terkini (via MarketDataPort)
        let current_tick = self.market_data.get_latest_tick(symbol).await?;
        let candles = self
            .market_data
            .get_recent_candles(symbol, timeframe, 100)
            .await?;

        if candles.is_empty() {
            warn!("Data candle kosong untuk {}", symbol);
            return Ok(None);
        }

        let ctx = MarketContext {
            symbol,
            timeframe,
            current_tick: &current_tick,
            candles: &candles,
            risk_profile: &self.risk_profile,
        };

        // 2. Evaluasi terhadap seluruh strategi terdaftar (Composition over Inheritance)
        for strategy in &self.strategies {
            if let Some(signal) = strategy.evaluate(&ctx).await? {
                info!(
                    "🎯 Sinyal terdeteksi dari strategi '{}' untuk {}",
                    strategy.name(),
                    symbol
                );

                // 3. Validasi Invariant Kepatuhan Regulasi Traders Family (Zero Penalty / Anti-Banned)
                if let Err(err) = TfComplianceGuard::validate_signal(&signal) {
                    warn!(
                        "⚠️ Sinyal untuk {} ditolak oleh TfComplianceGuard: {}",
                        symbol, err
                    );
                    continue;
                }

                // 4. Simpan ke database
                if let Err(e) = self.storage.save_signal(&signal).await {
                    error!("Gagal menyimpan sinyal ke database: {:?}", e);
                }

                // 5. Broadcast ke seluruh publisher (Trader Family, Telegram, dll)
                for publisher in &self.publishers {
                    match publisher.publish_signal(&signal).await {
                        Ok(receipt) => {
                            info!(
                                "✅ Sinyal berhasil dipublikasikan ke [{}] - ID: {}",
                                publisher.platform_name(),
                                receipt.external_post_id
                            );
                        }
                        Err(e) => {
                            error!(
                                "❌ Gagal mempublikasikan ke [{}]: {:?}",
                                publisher.platform_name(),
                                e
                            );
                        }
                    }
                }

                return Ok(Some(signal));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use broker_connector::BrokerConnector;
    use domain::models::{SignalAction, SignalStatus};
    use domain::ports::PublishReceipt;
    use rust_decimal_macros::dec;
    use storage_db::InMemoryStorage;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    struct MockStrategy;
    #[async_trait]
    impl StrategyPort for MockStrategy {
        fn name(&self) -> &str {
            "Mock-Strategy"
        }

        async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError> {
            Ok(Some(Signal {
                id: Uuid::new_v4(),
                symbol: ctx.symbol.clone(),
                action: SignalAction::BuyLimit,
                timeframe: ctx.timeframe,
                entry_price: dec!(1.08500),
                stop_loss: dec!(1.08300),     // 20 pips SL
                take_profit_1: dec!(1.08900), // 40 pips TP (R:R 1:2.0)
                take_profit_2: None,
                take_profit_3: None,
                risk_reward_ratio: dec!(2.0),
                confidence_score: 0.9,
                strategy_name: "Mock-Strategy".to_string(),
                rationale: "Unit test mock trigger".to_string(),
                status: SignalStatus::Pending,
                created_at: chrono::Utc::now(),
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            }))
        }
    }

    struct MockPublisher {
        published_count: Arc<Mutex<usize>>,
    }

    #[async_trait]
    impl SignalPublisherPort for MockPublisher {
        fn platform_name(&self) -> &'static str {
            "mock-tf"
        }

        async fn publish_signal(&self, signal: &Signal) -> Result<PublishReceipt, DomainError> {
            let mut count = self.published_count.lock().await;
            *count += 1;
            Ok(PublishReceipt {
                signal_id: signal.id,
                channel_target: "test_channel".to_string(),
                external_post_id: "mock_post_123".to_string(),
                published_at: chrono::Utc::now(),
            })
        }

        async fn update_signal_status(
            &self,
            _receipt: &PublishReceipt,
            _updated_signal: &Signal,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_signal_engine_di_orchestration() {
        let market_data = Arc::new(BrokerConnector::new("MockBroker"));
        let published_counter = Arc::new(Mutex::new(0));
        let mock_publisher: Arc<dyn SignalPublisherPort> = Arc::new(MockPublisher {
            published_count: published_counter.clone(),
        });
        let storage = Arc::new(InMemoryStorage::new());
        let strategy: Arc<dyn StrategyPort> = Arc::new(MockStrategy);

        let engine = SignalEngineService::new(
            market_data,
            vec![mock_publisher],
            storage.clone(),
            vec![strategy],
            RiskProfile::default(),
        );

        let symbol = Symbol::new("EUR", "USD");
        let signal_opt = engine.process_symbol(&symbol, Timeframe::M15).await.unwrap();

        assert!(signal_opt.is_some());
        let signal = signal_opt.unwrap();
        assert_eq!(signal.strategy_name, "Mock-Strategy");

        // Verifikasi publisher terpanggil 1x
        let count = *published_counter.lock().await;
        assert_eq!(count, 1);

        // Verifikasi data tersimpan di storage
        let saved = storage.get_signal(signal.id).await.unwrap();
        assert!(saved.is_some());
    }

    #[tokio::test]
    async fn test_signal_engine_with_pola_n_strategy() {
        use domain::models::PolaNStrategy;
        let market_data = Arc::new(BrokerConnector::new("MetaTrader5"));
        let storage = Arc::new(InMemoryStorage::new());
        let pola_n: Arc<dyn StrategyPort> = Arc::new(PolaNStrategy::default());

        let engine = SignalEngineService::new(
            market_data,
            vec![],
            storage,
            vec![pola_n],
            RiskProfile::default(),
        );

        let symbol = Symbol::new("EUR", "USD");
        let result = engine.process_symbol(&symbol, Timeframe::M15).await;
        assert!(result.is_ok());
    }
}
