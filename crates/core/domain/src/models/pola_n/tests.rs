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
        let pip_size = dec!(0.00010); // Standard 5-digit forex pip size
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
                // L2 = 65% retracement dari impulse 100 pips → 1.1100 - 0.0065 = 1.1035
                // Berada dalam window 38%-85% (retracement = 65%)
                price: dec!(1.1035),
                is_high: false,
            },
        ];

        let result = engine.evaluate_swings(&swings, dec!(1.1040), pip_size);
        assert!(
            result.is_some(),
            "Formasi bullish pada 65% retracement harus terdeteksi"
        );
        let formation = result.unwrap();
        assert_eq!(formation.pattern_type, PolaNType::BullishN);
        assert_eq!(formation.point_1, dec!(1.1000));
        assert_eq!(formation.point_2, dec!(1.1100));
        assert_eq!(formation.point_3, dec!(1.1035));
        // Golden Pullback Entry: L2 + 25% of pullback = 1.1035 + 0.001625 = 1.105125
        assert_eq!(formation.suggested_entry, dec!(1.105125));
        // SL Struktural: L2 - 2 pip = 1.1035 - 0.0002 = 1.1033
        assert_eq!(formation.stop_loss, dec!(1.1033));
        assert_eq!(formation.risk_reward_ratio, dec!(1.49));
    }

    #[test]
    fn test_bearish_pola_n_detection() {
        let engine = PolaNFormationEngine::default();
        let pip_size = dec!(0.00010);
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
                // H2 = 65% retracement dari impulse 100 pips → 1.1900 + 0.0065 = 1.1965
                // Berada dalam window 38%-85% (retracement = 65%)
                price: dec!(1.1965),
                is_high: true,
            },
        ];

        let result = engine.evaluate_swings(&swings, dec!(1.1960), pip_size);
        assert!(
            result.is_some(),
            "Formasi bearish pada 65% retracement harus terdeteksi"
        );
        let formation = result.unwrap();
        assert_eq!(formation.pattern_type, PolaNType::BearishN);
        assert_eq!(formation.point_1, dec!(1.2000));
        assert_eq!(formation.point_2, dec!(1.1900));
        assert_eq!(formation.point_3, dec!(1.1965));
        // Golden Pullback Entry: H2 - 25% of pullback = 1.1965 - 0.001625 = 1.194875
        assert_eq!(formation.suggested_entry, dec!(1.194875));
        // SL Struktural: H2 + 2 pip = 1.1965 + 0.0002 = 1.1967
        assert_eq!(formation.stop_loss, dec!(1.1967));
        assert_eq!(formation.risk_reward_ratio, dec!(1.49));
    }

    #[test]
    fn test_invalid_pola_n_rejected() {
        let engine = PolaNFormationEngine::default();
        let pip_size = dec!(0.00010);
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

        let result = engine.evaluate_swings(&invalid_swings, dec!(1.0960), pip_size);
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
