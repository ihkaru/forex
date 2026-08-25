use chrono::{DateTime, TimeZone, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, MarketDataSource, PolaNStrategy, RiskProfile, Symbol, Timeframe};
use domain::ports::{MarketDataPort, StrategyPort, TfQualificationAuditPort};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

use application::services::TfQualificationService;

struct MockMarketFeed {
    candles: Vec<Candle>,
}

#[async_trait::async_trait]
impl MarketDataPort for MockMarketFeed {
    fn source(&self) -> MarketDataSource {
        MarketDataSource::SyntheticTest
    }

    async fn get_latest_tick(&self, _symbol: &Symbol) -> Result<domain::models::Tick, DomainError> {
        Ok(domain::models::Tick {
            symbol: Symbol::new("XAU", "USD"),
            timestamp: Utc::now(),
            source: MarketDataSource::SyntheticTest,
            bid: dec!(2000.0),
            ask: dec!(2000.3),
        })
    }

    async fn get_recent_candles(
        &self,
        _symbol: &Symbol,
        _timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let n = self.candles.len();
        let start = n.saturating_sub(limit);
        Ok(self.candles[start..].to_vec())
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

fn generate_deterministic_synthetic_candles() -> Vec<Candle> {
    let mut candles = Vec::new();
    let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let mut current_price = dec!(2000.0);
    let sym = Symbol::new("XAU", "USD");

    for i in 0..600 {
        let time = start + chrono::Duration::hours(i);
        let cycle = i % 20;
        let delta = if cycle < 10 {
            dec!(2.5)
        } else if cycle < 15 {
            dec!(-1.5)
        } else {
            dec!(3.0)
        };

        let open = current_price;
        let close = open + delta;
        let high = open.max(close) + dec!(1.0);
        let low = open.min(close) - dec!(1.0);
        current_price = close;

        candles.push(Candle {
            symbol: sym.clone(),
            timeframe: Timeframe::H1,
            timestamp: time,
            source: MarketDataSource::SyntheticTest,
            open,
            high,
            low,
            close,
            volume: dec!(1500.0),
        });
    }

    candles
}

#[tokio::test]
async fn test_tf_qualification_audit_service_deterministic_pipeline() {
    let mock_candles = generate_deterministic_synthetic_candles();
    let market_feed = Arc::new(MockMarketFeed {
        candles: mock_candles,
    });

    let service = TfQualificationService::new(market_feed, RiskProfile::default());
    let strategy: Arc<dyn StrategyPort> = Arc::new(PolaNStrategy::with_params(
        "Test-Audit-Strategy",
        3,
        2,
        dec!(0.00020),
        dec!(1.20),
    ));

    let from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();

    let audit_result = service.audit_strategy(strategy, from, to).await;
    assert!(audit_result.is_ok(), "Audit strategy must return Ok");

    let report = audit_result.unwrap();
    assert_eq!(report.strategy_name, "Test-Audit-Strategy");
    assert_eq!(
        report.hard_invariants.len(),
        8,
        "Must evaluate all 8 hard invariants"
    );
    assert!(
        report.all_invariants_passed,
        "All 8 invariants must pass deterministically"
    );
}

#[test]
fn test_tiered_pips_reward_calculation_formula() {
    // 1. Unqualified or VP < 300 -> 0 Points
    let (_, _, pts_unqualified) =
        TfQualificationService::calculate_tiered_points(dec!(250.0), dec!(0.5), false);
    assert_eq!(pts_unqualified, Decimal::ZERO);

    // 2. Qualified with exactly 500 VP at Legend Level (0.5x Multiplier)
    // Tier 2: 500 * 0.5 = 250 TF Points, Tier 3: 0
    let (t2, t3, total) =
        TfQualificationService::calculate_tiered_points(dec!(500.0), dec!(0.5), true);
    assert_eq!(t2, dec!(250.0));
    assert_eq!(t3, dec!(0.0));
    assert_eq!(total, dec!(250.0));

    // 3. Qualified with 600 VP at Legend Level (0.5x Multiplier)
    // Tier 2: 500 * 0.5 = 250 TF Points
    // Tier 3: 100 * 0.5 * 20% = 10 TF Points
    // Total = 260 TF Points
    let (t2_surplus, t3_surplus, total_surplus) =
        TfQualificationService::calculate_tiered_points(dec!(600.0), dec!(0.5), true);
    assert_eq!(t2_surplus, dec!(250.0));
    assert_eq!(t3_surplus, dec!(10.0));
    assert_eq!(total_surplus, dec!(260.0));
}

#[test]
fn test_career_medals_and_multiplier_progression() {
    assert_eq!(
        TfQualificationService::get_level_info(0),
        ("Newbie", dec!(0.0))
    );
    assert_eq!(
        TfQualificationService::get_level_info(1),
        ("Rookie", dec!(1.0))
    );
    assert_eq!(
        TfQualificationService::get_level_info(2),
        ("Rookie", dec!(1.0))
    );
    assert_eq!(
        TfQualificationService::get_level_info(3),
        ("Pro", dec!(0.2))
    );
    assert_eq!(
        TfQualificationService::get_level_info(4),
        ("Pro", dec!(0.2))
    );
    assert_eq!(
        TfQualificationService::get_level_info(5),
        ("Elite", dec!(0.2))
    );
    assert_eq!(
        TfQualificationService::get_level_info(7),
        ("Elite", dec!(0.2))
    );
    assert_eq!(
        TfQualificationService::get_level_info(8),
        ("Master", dec!(0.3))
    );
    assert_eq!(
        TfQualificationService::get_level_info(10),
        ("Master", dec!(0.3))
    );
    assert_eq!(
        TfQualificationService::get_level_info(11),
        ("Legend", dec!(0.5))
    );
    assert_eq!(
        TfQualificationService::get_level_info(25),
        ("Legend", dec!(0.5))
    );
}
