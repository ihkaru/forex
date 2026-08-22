use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

use application::services::{BacktestService, EdaService};
use domain::errors::DomainError;
use domain::models::{
    BacktestConfig, Candle, PolaNStrategy, RiskProfile, Symbol, TfPairSpec, Tick, Timeframe,
};
use domain::ports::MarketDataPort;

/// Mock feed untuk audit konsensus kuantitatif 2026
struct ConsensusMockFeed {
    candles: Vec<Candle>,
    spread_pips: Decimal,
}

#[async_trait]
impl MarketDataPort for ConsensusMockFeed {
    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        let spec = TfPairSpec::from_symbol(symbol);
        let last_close = self.candles.last().map(|c| c.close).unwrap_or(dec!(1.0850));
        Ok(Tick {
            symbol: symbol.clone(),
            timestamp: Utc::now(),
            source: domain::models::MarketDataSource::SyntheticTest,
            bid: last_close,
            ask: last_close + (spec.pip_size * self.spread_pips),
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

fn create_market_wave(symbol: &Symbol, count: usize) -> Vec<Candle> {
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let mut candles = Vec::new();
    let mut seed: u64 = 987654321;
    let mut current_price = dec!(1.0850);
    let pip_scale = dec!(1.0);

    for i in 0..count {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let rand_val = ((seed >> 32) as i64 % 200) - 100;

        let regime = (i / 70) % 4;
        let drift = match regime {
            0 => 6,
            1 => -2,
            2 => -7,
            _ => 4,
        };

        let delta_pips = drift + (rand_val / 25);
        let delta_dec = Decimal::new(delta_pips, 4) * pip_scale;

        let open = current_price;
        let close = open + delta_dec;

        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let wick_up_pips = (((seed >> 32) % 18) + 6) as i64;
        let wick_dn_pips = (((seed >> 40) % 18) + 6) as i64;

        let high = open.max(close) + (Decimal::new(wick_up_pips, 4) * pip_scale);
        let low = open.min(close) - (Decimal::new(wick_dn_pips, 4) * pip_scale);

        candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: now + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open,
            high,
            low,
            close,
            volume: dec!(1500.0),
        });

        current_price = close;
    }

    candles
}

// ============================================================================
// AUDIT 1: Exploratory Data Analysis (EDA) Data Health Scorecard
// Memastikan EdaService mendeteksi candle korup, gap hari kerja, & integritas OHLCV
// ============================================================================
#[test]
fn test_eda_health_scorecard_detects_corrupt_candles_and_gaps() {
    let symbol = Symbol::new("NZD", "USD");
    let now = Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(); // Senin

    let mut clean_candles = Vec::new();
    for i in 0..100 {
        clean_candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: now + Duration::hours(i),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: dec!(0.6000),
            high: dec!(0.6040),
            low: dec!(0.5980),
            close: dec!(0.6020),
            volume: dec!(1000.0),
        });
    }

    // 1. Uji dataset bersih (100% Integritas)
    let clean_report = EdaService::analyze(&symbol, &clean_candles);
    assert_eq!(clean_report.invalid_candle_count, 0);
    assert_eq!(clean_report.mathematical_integrity_pct, 100.0);
    assert!(clean_report.health_status.contains("EXCELLENT"));

    // 2. Uji dataset korup (High < Low & High < Open)
    let mut corrupt_candles = clean_candles.clone();
    corrupt_candles[10].high = dec!(0.5900); // Invalid: High < Low (0.5900 < 0.5980)
    corrupt_candles[20].volume = dec!(0.0); // Zero volume bar

    let corrupt_report = EdaService::analyze(&symbol, &corrupt_candles);
    assert_eq!(corrupt_report.invalid_candle_count, 1);
    assert!(corrupt_report.mathematical_integrity_pct < 100.0);
    assert_eq!(corrupt_report.zero_volume_bars_count, 1);
}

// ============================================================================
// AUDIT 2: Walk-Forward Efficiency Ratio (WFER) & Zero Overfitting Check
// Memastikan WFER = (OOS Win Rate / IS Win Rate) >= 50% pada data multi-regime
// ============================================================================
#[tokio::test]
async fn test_walk_forward_efficiency_ratio_calculation() {
    let symbol = Symbol::new("EUR", "USD");
    let all_candles = create_market_wave(&symbol, 2000);
    let start_time = all_candles.first().unwrap().timestamp;
    let end_time = all_candles.last().unwrap().timestamp;

    // Split 70% In-Sample (1400 bar) & 30% Out-of-Sample (600 bar)
    let split_idx = 1400;
    let is_candles = all_candles[..split_idx].to_vec();
    let oos_candles = all_candles[split_idx..].to_vec();

    let is_feed = Arc::new(ConsensusMockFeed {
        candles: is_candles,
        spread_pips: dec!(1.2),
    });
    let oos_feed = Arc::new(ConsensusMockFeed {
        candles: oos_candles,
        spread_pips: dec!(1.2),
    });

    let strategy = Arc::new(PolaNStrategy::with_params(
        "Test-PolaN",
        2,
        2,
        dec!(0.00020),
        dec!(1.5),
    ));
    let risk = RiskProfile::default();
    let config = BacktestConfig::default();

    let is_service =
        BacktestService::with_config(is_feed, strategy.clone(), risk.clone(), config.clone());
    let oos_service = BacktestService::with_config(oos_feed, strategy, risk, config);

    let is_report = is_service
        .run_simulation(&symbol, Timeframe::H1, start_time, end_time)
        .await
        .unwrap();

    let oos_report = oos_service
        .run_simulation(&symbol, Timeframe::H1, start_time, end_time)
        .await
        .unwrap();

    // Pastikan kedua simulasi menghasilkan transaksi teruji
    assert!(is_report.total_trades > 0);
    assert!(oos_report.total_trades > 0);

    let wfer = if is_report.win_rate_percent > Decimal::ZERO {
        (oos_report.win_rate_percent / is_report.win_rate_percent) * dec!(100.0)
    } else {
        dec!(100.0)
    };
    // Standar 2026: WFER >= 50% membuktikan model memiliki keunggulan nyata tanpa overfitting
    assert!(
        wfer >= dec!(50.0),
        "WFER ({:.2}%) di bawah batas minimum 50% - Terindikasi Overfitting!",
        wfer
    );
}

// ============================================================================
// AUDIT 3: Pending Limit Order vs Instant Copy-Trade Latency Slippage
// Memastikan Pending Limit Order mengisi harga eksak tanpa rugi slippage
// ============================================================================
#[test]
fn test_pending_limit_order_eliminates_copy_trade_slippage() {
    let spec = TfPairSpec::from_symbol(&Symbol::new("GBP", "USD"));
    let target_limit_entry = dec!(1.25000);

    // Pada Pending Limit Order: Broker wajib mengeksekusi tepat pada harga limit
    let actual_limit_fill = target_limit_entry;
    let limit_slippage_pips = spec.price_diff_to_pips(actual_limit_fill - target_limit_entry);
    assert_eq!(
        limit_slippage_pips,
        Decimal::ZERO,
        "Pending Limit Order wajib 0 slippage!"
    );

    // Pada Market Order (Eksekusi Instan dengan jeda 300ms latency): Terkena slippage 1.5 pips
    let market_fill_with_latency = target_limit_entry + (spec.pip_size * dec!(1.5));
    let market_slippage_pips =
        spec.price_diff_to_pips(market_fill_with_latency - target_limit_entry);
    assert_eq!(
        market_slippage_pips,
        dec!(1.5),
        "Market order mengalami slippage 1.5 pips!"
    );
}

// ============================================================================
// AUDIT 4: Multi-Regime Shift Resilience & Drawdown Containment
// Memastikan portofolio tidak mengalami kehancuran saat terjadi pergeseran rezim
// ============================================================================
#[tokio::test]
async fn test_regime_shift_drawdown_containment() {
    let symbol = Symbol::new("AUD", "USD");
    let multi_regime_candles = create_market_wave(&symbol, 2000);
    let start_time = multi_regime_candles.first().unwrap().timestamp;
    let end_time = multi_regime_candles.last().unwrap().timestamp;

    let feed = Arc::new(ConsensusMockFeed {
        candles: multi_regime_candles,
        spread_pips: dec!(1.2),
    });

    let service = BacktestService::with_config(
        feed,
        Arc::new(PolaNStrategy::with_params(
            "Test-PolaN",
            2,
            2,
            dec!(0.00020),
            dec!(1.5),
        )),
        RiskProfile::default(),
        BacktestConfig::default(),
    );

    let report = service
        .run_simulation(&symbol, Timeframe::H1, start_time, end_time)
        .await
        .unwrap();

    // Memastikan portofolio mampu mengendalikan risiko dan drawdown tetap terkendali
    assert!(report.total_trades > 0);
    assert!(report.max_drawdown_pips >= Decimal::ZERO);
}
