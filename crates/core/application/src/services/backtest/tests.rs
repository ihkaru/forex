#[cfg(test)]
#[allow(clippy::module_inception)] // Idiom Rust standard untuk test module di file terpisah
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
        ticks: Vec<Tick>,
    }

    impl MockMarketData {
        fn new(candles: Vec<Candle>) -> Self {
            Self {
                candles,
                ticks: Vec::new(),
            }
        }

        fn with_ticks(candles: Vec<Candle>, ticks: Vec<Tick>) -> Self {
            Self { candles, ticks }
        }
    }

    #[async_trait]
    impl MarketDataPort for MockMarketData {
        fn source(&self) -> domain::models::MarketDataSource {
            domain::models::MarketDataSource::SyntheticTest
        }

        async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
            Ok(Tick {
                symbol: symbol.clone(),
                timestamp: Utc::now(),
                source: domain::models::MarketDataSource::SyntheticTest,
                bid: dec!(1.0850),
                ask: dec!(1.0852),
                spread_pts: None,
                volume: None,
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

        async fn get_historical_ticks(
            &self,
            _symbol: &Symbol,
            from: DateTime<Utc>,
            to: DateTime<Utc>,
        ) -> Result<Vec<Tick>, DomainError> {
            Ok(self
                .ticks
                .iter()
                .filter(|t| t.timestamp >= from && t.timestamp <= to)
                .cloned()
                .collect())
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
                source: domain::models::MarketDataSource::SyntheticTest,
                open: dec!(0.6000) + Decimal::new(i as i64 * 5, 4),
                high: dec!(0.6050) + Decimal::new(i as i64 * 5, 4),
                low: dec!(0.5980) + Decimal::new(i as i64 * 5, 4),
                close: dec!(0.6020) + Decimal::new(i as i64 * 5, 4),
                volume: dec!(100.0),
                spread: None,
                real_volume: None,
            });
        }

        let market_data = Arc::new(MockMarketData::new(candles));
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

    #[tokio::test]
    async fn test_backtest_with_commission_and_slippage() {
        use domain::models::BacktestConfig;

        let symbol = Symbol::new("EUR", "USD");
        let now = Utc::now();
        let mut candles = Vec::new();

        for i in 0..80 {
            candles.push(Candle {
                symbol: symbol.clone(),
                timeframe: Timeframe::H1,
                timestamp: now + chrono::Duration::hours(i as i64),
                source: domain::models::MarketDataSource::SyntheticTest,
                open: dec!(1.0800) + Decimal::new(i as i64 * 3, 4),
                high: dec!(1.0860) + Decimal::new(i as i64 * 3, 4),
                low: dec!(1.0770) + Decimal::new(i as i64 * 3, 4),
                close: dec!(1.0820) + Decimal::new(i as i64 * 3, 4),
                volume: dec!(100.0),
                spread: None,
                real_volume: None,
            });
        }

        let config = BacktestConfig {
            simulation_spread_pips: dec!(1.0),
            lookback_window_bars: 50,
            default_lot_size: dec!(0.10),
            enable_swap: false,
            enforce_stop_levels: false,
            enable_slippage: true,
            slippage_pips: dec!(0.5),
            commission_per_lot_pips: dec!(0.8),
        };

        let market_data = Arc::new(MockMarketData::new(candles));
        let strategy = Arc::new(PolaNStrategy::default());
        let service = BacktestService::with_config(market_data, strategy, RiskProfile::default(), config);

        let report = service
            .run_simulation(
                &symbol,
                Timeframe::H1,
                now,
                now + chrono::Duration::hours(80),
            )
            .await
            .unwrap();

        assert_eq!(report.symbol.to_compact_string(), "EURUSD");
        // Jika ada trade selesai, periksa pemotongan friksi
        if report.total_trades > 0 {
            assert!(report.total_commission_deducted_pips >= Decimal::ZERO);
            assert!(report.total_slippage_deducted_pips >= Decimal::ZERO);
        }
    }

    #[tokio::test]
    async fn test_backtest_tick_walk_dual_resolution() {
        let symbol = Symbol::new("EUR", "USD");
        let now = Utc::now();
        let mut candles = Vec::new();
        let mut ticks = Vec::new();

        for i in 0..80 {
            let t = now + chrono::Duration::hours(i as i64);
            candles.push(Candle {
                symbol: symbol.clone(),
                timeframe: Timeframe::H1,
                timestamp: t,
                source: domain::models::MarketDataSource::SyntheticTest,
                open: dec!(1.0800) + Decimal::new(i as i64 * 3, 4),
                high: dec!(1.0860) + Decimal::new(i as i64 * 3, 4),
                low: dec!(1.0770) + Decimal::new(i as i64 * 3, 4),
                close: dec!(1.0820) + Decimal::new(i as i64 * 3, 4),
                volume: dec!(100.0),
                spread: None,
                real_volume: None,
            });

            // Tambahkan tick mikro-struktur untuk 10 bar terakhir
            if i >= 70 {
                ticks.push(Tick {
                    symbol: symbol.clone(),
                    timestamp: t + chrono::Duration::minutes(15),
                    source: domain::models::MarketDataSource::SyntheticTest,
                    bid: dec!(1.0810) + Decimal::new(i as i64 * 3, 4),
                    ask: dec!(1.0812) + Decimal::new(i as i64 * 3, 4),
                    spread_pts: Some(2),
                    volume: None,
                });
            }
        }

        let market_data = Arc::new(MockMarketData::with_ticks(candles, ticks));
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

        assert_eq!(report.symbol.to_compact_string(), "EURUSD");
        assert!(report.execution_model.contains("DIRECTIONAL_4POINT") || report.execution_model.contains("DUAL_RESOLUTION"));
    }
}
