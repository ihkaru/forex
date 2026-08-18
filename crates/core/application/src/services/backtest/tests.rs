#[cfg(test)]
mod tests {
    use super::super::service::BacktestService;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use domain::errors::DomainError;
    use domain::models::{Candle, PolaNStrategy, RiskProfile, Symbol, Tick, Timeframe};
    use domain::ports::MarketDataPort;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    struct MockMarketData {
        candles: Vec<Candle>,
    }

    #[async_trait]
    impl MarketDataPort for MockMarketData {
        async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
            Ok(Tick {
                symbol: symbol.clone(),
                timestamp: Utc::now(),
                bid: dec!(1.0850),
                ask: dec!(1.0852),
            })
        }

        async fn get_recent_candles(
            &self,
            _symbol: &Symbol,
            _timeframe: Timeframe,
            _limit: usize,
        ) -> Result<Vec<Candle>, DomainError> {
            Ok(self.candles.clone())
        }

        async fn get_historical_candles(
            &self,
            _symbol: &Symbol,
            _timeframe: Timeframe,
            _from: DateTime<Utc>,
            _to: DateTime<Utc>,
        ) -> Result<Vec<Candle>, DomainError> {
            Ok(self.candles.clone())
        }
    }

    #[tokio::test]
    async fn test_backtest_simulation_computes_tf_metrics() {
        let symbol = Symbol::new("NZD", "USD");
        let now = Utc::now();
        let mut candles = Vec::new();

        // Buat 80 candle untuk simulasi
        for i in 0..80 {
            candles.push(Candle {
                symbol: symbol.clone(),
                timeframe: Timeframe::H1,
                timestamp: now + chrono::Duration::hours(i as i64),
                open: dec!(0.6000) + Decimal::new(i as i64 * 5, 4),
                high: dec!(0.6050) + Decimal::new(i as i64 * 5, 4),
                low: dec!(0.5980) + Decimal::new(i as i64 * 5, 4),
                close: dec!(0.6020) + Decimal::new(i as i64 * 5, 4),
                volume: dec!(100.0),
            });
        }

        let market_data = Arc::new(MockMarketData { candles });
        let strategy = Arc::new(PolaNStrategy::default());
        let service = BacktestService::new(market_data, strategy, RiskProfile::default());

        let report = service
            .run_simulation(
                &symbol,
                Timeframe::H1,
                now,
                now + chrono::Duration::hours(80),
            )
            .await
            .unwrap();

        assert_eq!(report.symbol.to_compact_string(), "NZDUSD");
        assert!(report.profit_factor >= Decimal::ZERO);
    }
}
