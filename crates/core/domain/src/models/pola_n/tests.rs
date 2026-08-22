#[cfg(test)]
#[allow(clippy::module_inception)] // Idiom Rust standard untuk test module di file terpisah
mod tests {
    use crate::models::pola_n::{PolaNFormationEngine, PolaNStrategy, PolaNType, SwingPoint};
    use crate::models::{Candle, RiskProfile, Symbol, Tick, Timeframe};
    use crate::ports::{MarketContext, StrategyPort};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn mock_candle(time_offset_hours: i64, o: f64, h: f64, l: f64, c: f64) -> Candle {
        Candle {
            symbol: Symbol::new("EUR", "USD"),
            timeframe: Timeframe::H1,
            timestamp: Utc::now() + chrono::Duration::hours(time_offset_hours),
            source: crate::models::MarketDataSource::SyntheticTest,
            open: dec!(1.0) * Decimal::from_f64_retain(o).unwrap(),
            high: dec!(1.0) * Decimal::from_f64_retain(h).unwrap(),
            low: dec!(1.0) * Decimal::from_f64_retain(l).unwrap(),
            close: dec!(1.0) * Decimal::from_f64_retain(c).unwrap(),
            volume: dec!(100),
        }
    }

    #[test]
    fn test_bullish_pola_n_detection() {
        let engine = PolaNFormationEngine::default();
        let swings = vec![
            SwingPoint {
                index: 0,
                price: dec!(1.1000), // L1
                is_high: false,
            },
            SwingPoint {
                index: 10,
                price: dec!(1.1100), // H1 (+100 pips)
                is_high: true,
            },
            SwingPoint {
                index: 15,
                price: dec!(1.1040), // L2 Higher Low (Retest 60% retracement)
                is_high: false,
            },
        ];

        let result = engine.evaluate_swings(&swings, dec!(1.1045));
        assert!(result.is_some());
        let formation = result.unwrap();
        assert_eq!(formation.pattern_type, PolaNType::BullishN);
        assert_eq!(formation.point_1, dec!(1.1000));
        assert_eq!(formation.point_2, dec!(1.1100));
        assert_eq!(formation.point_3, dec!(1.1040));
        assert_eq!(formation.take_profit_1, dec!(1.11240));
        assert_eq!(formation.risk_reward_ratio, dec!(2.0));
    }

    #[test]
    fn test_bearish_pola_n_detection() {
        let engine = PolaNFormationEngine::default();
        let swings = vec![
            SwingPoint {
                index: 0,
                price: dec!(1.2000), // H1
                is_high: true,
            },
            SwingPoint {
                index: 10,
                price: dec!(1.1900), // L1 (-100 pips)
                is_high: false,
            },
            SwingPoint {
                index: 15,
                price: dec!(1.1960), // H2 Lower High (Retest 40% discount)
                is_high: true,
            },
        ];

        let result = engine.evaluate_swings(&swings, dec!(1.1955));
        assert!(result.is_some());
        let formation = result.unwrap();
        assert_eq!(formation.pattern_type, PolaNType::BearishN);
        assert_eq!(formation.point_1, dec!(1.2000));
        assert_eq!(formation.point_2, dec!(1.1900));
        assert_eq!(formation.point_3, dec!(1.1960));
        assert_eq!(formation.take_profit_1, dec!(1.18760));
        assert_eq!(formation.risk_reward_ratio, dec!(2.0));
    }

    #[test]
    fn test_invalid_pola_n_rejected() {
        let engine = PolaNFormationEngine::default();
        // L2 lebih rendah dari L1 (Bukan Higher Low, melainkan Lower Low -> Gagal Struktur Pola N)
        let invalid_swings = vec![
            SwingPoint {
                index: 0,
                price: dec!(1.1000), // L1
                is_high: false,
            },
            SwingPoint {
                index: 10,
                price: dec!(1.1100), // H1
                is_high: true,
            },
            SwingPoint {
                index: 15,
                price: dec!(1.0950), // L2 < L1 (Invalid breakdown)
                is_high: false,
            },
        ];

        let result = engine.evaluate_swings(&invalid_swings, dec!(1.0960));
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_pola_n_strategy_evaluation() {
        let strategy = PolaNStrategy::default();
        let sym = Symbol::new("EUR", "USD");
        let risk = RiskProfile::default();

        let mut candles = Vec::new();
        for i in 0..50 {
            candles.push(mock_candle(i, 1.1000, 1.1010, 1.0990, 1.1005));
        }
        candles.push(mock_candle(51, 1.0950, 1.0960, 1.0940, 1.0945));
        for i in 52..65 {
            candles.push(mock_candle(
                i,
                1.0950 + (i as f64 - 51.0) * 0.001,
                1.0960 + (i as f64 - 51.0) * 0.001,
                1.0945 + (i as f64 - 51.0) * 0.001,
                1.0955 + (i as f64 - 51.0) * 0.001,
            ));
        }

        let tick = Tick {
            symbol: sym.clone(),
            bid: dec!(1.1050),
            ask: dec!(1.1052),
            source: crate::models::MarketDataSource::SyntheticTest,
            timestamp: Utc::now(),
        };

        let ctx = MarketContext {
            symbol: &sym,
            timeframe: Timeframe::H1,
            current_tick: &tick,
            candles: &candles,
            risk_profile: &risk,
        };

        let eval = strategy.evaluate(&ctx).await;
        assert!(eval.is_ok());
    }
}
