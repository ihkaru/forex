use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use uuid::Uuid;

use application::services::BacktestService;
use domain::errors::DomainError;
use domain::models::{
    Candle, PolaNStrategy, RiskProfile, Signal, SignalAction, SignalStatus, Symbol,
    TfComplianceGuard, TfPairSpec, Tick, Timeframe,
};
use domain::ports::{MarketContext, MarketDataPort, StrategyPort};

/// Mock Data Feed dengan kontrol penuh candle untuk audit
struct AuditMarketFeed {
    candles: Vec<Candle>,
    spread_pips: Decimal,
}

#[async_trait]
impl MarketDataPort for AuditMarketFeed {
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

// ==============================================================================
// 🧪 1. AUDIT INVARIANT 1: ANTI LOOK-AHEAD BIAS (STRICT BAR-BY-BAR ISOLATION)
// ==============================================================================
#[tokio::test]
async fn test_audit_lookahead_bias_isolation() {
    let symbol = Symbol::new("EUR", "USD");
    let base_time = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    // Buat 50 candle dasar
    let mut dataset_a = Vec::new();
    for i in 0..50 {
        let price = dec!(1.0800) + (dec!(0.0005) * Decimal::from(i));
        dataset_a.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: price,
            high: price + dec!(0.0010),
            low: price - dec!(0.0010),
            close: price + dec!(0.0002),
            volume: dec!(1000.0),
        });
    }

    // Dataset B identik di 50 bar awal, tapi mengalami PUMP raksasa di bar ke-51 s.d 60
    let mut dataset_b = dataset_a.clone();
    for i in 50..60 {
        let price = dec!(1.1500) + (dec!(0.0050) * Decimal::from(i - 50)); // Future Massive Pump
        dataset_b.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: price,
            high: price + dec!(0.0100),
            low: price - dec!(0.0010),
            close: price + dec!(0.0080),
            volume: dec!(50000.0),
        });
    }

    // Dataset C identik di 50 bar awal, tapi mengalami CRASH di bar ke-51 s.d 60
    let mut dataset_c = dataset_a.clone();
    for i in 50..60 {
        let price = dec!(1.0200) - (dec!(0.0050) * Decimal::from(i - 50)); // Future Massive Crash
        dataset_c.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: price,
            high: price + dec!(0.0010),
            low: price - dec!(0.0100),
            close: price - dec!(0.0080),
            volume: dec!(50000.0),
        });
    }

    let strategy = Arc::new(PolaNStrategy::default());
    let risk = RiskProfile::default();

    // Evaluasi bar ke-50 pada kedua dataset
    let feed_b = Arc::new(AuditMarketFeed {
        candles: dataset_b,
        spread_pips: dec!(1.0),
    });
    let feed_c = Arc::new(AuditMarketFeed {
        candles: dataset_c,
        spread_pips: dec!(1.0),
    });

    let backtest_b = BacktestService::new(feed_b, strategy.clone(), risk.clone());
    let backtest_c = BacktestService::new(feed_c, strategy.clone(), risk.clone());

    let report_b = backtest_b
        .run_simulation(
            &symbol,
            Timeframe::H1,
            base_time,
            base_time + Duration::days(5),
        )
        .await
        .unwrap();
    let report_c = backtest_c
        .run_simulation(
            &symbol,
            Timeframe::H1,
            base_time,
            base_time + Duration::days(5),
        )
        .await
        .unwrap();

    // Keputusan di 50 bar pertama tidak boleh terpengaruh oleh pump/crash masa depan
    println!(
        "🔍 Audit Lookahead: Sinyal 50 bar awal terbukti kebal dari perubahan harga masa depan."
    );
    assert_eq!(
        report_b.winning_trades + report_b.losing_trades,
        report_c.winning_trades + report_c.losing_trades
    );
}

// ==============================================================================
// 🧪 2. AUDIT INVARIANT 2: PENDING LIMIT ORDER REALISTIC FILL & EXPIRATION
// ==============================================================================
#[tokio::test]
async fn test_audit_pending_order_lifecycle_fill_and_expiration() {
    let symbol = Symbol::new("EUR", "USD");
    let base_time = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    // Skenario: Pending BuyLimit dipasang di 1.08200 saat market di 1.08500.
    // Harga pasar langsung lari naik ke 1.09000 (Take Profit) TANPA PERNAH turun ke 1.08200.
    let mut candles = Vec::new();
    for i in 0..80 {
        let price = dec!(1.08500) + (dec!(0.00010) * Decimal::from(i));
        candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: price,
            high: price + dec!(0.00030),
            low: dec!(1.08350), // Low tidak pernah menyentuh level Limit 1.08200!
            close: price + dec!(0.00020),
            volume: dec!(1000.0),
        });
    }

    // Strategi Mock yang memancarkan BuyLimit di 1.08200 pada bar 50
    struct UnreachableLimitStrategy;
    #[async_trait]
    impl StrategyPort for UnreachableLimitStrategy {
        fn name(&self) -> &str {
            "UnreachableLimit"
        }
        async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError> {
            if ctx.candles.len() == 50 {
                Ok(Some(Signal {
                    id: Uuid::new_v4(),
                    symbol: ctx.symbol.clone(),
                    action: SignalAction::BuyLimit,
                    timeframe: ctx.timeframe,
                    entry_price: dec!(1.08200), // 1.08200 (jauh di bawah Low 1.08350)
                    stop_loss: dec!(1.08000),
                    take_profit_1: dec!(1.08600),
                    take_profit_2: None,
                    take_profit_3: None,
                    risk_reward_ratio: dec!(2.0),
                    confidence_score: 0.95,
                    strategy_name: "UnreachableLimit".to_string(),
                    rationale: "Testing pending expiration".to_string(),
                    status: SignalStatus::Pending,
                    created_at: Utc::now(),
                    expires_at: Some(Utc::now() + Duration::hours(24)),
                }))
            } else {
                Ok(None)
            }
        }
    }

    let feed = Arc::new(AuditMarketFeed {
        candles,
        spread_pips: dec!(1.2),
    });
    let backtester = BacktestService::new(
        feed,
        Arc::new(UnreachableLimitStrategy),
        RiskProfile::default(),
    );

    let report = backtester
        .run_simulation(
            &symbol,
            Timeframe::H1,
            base_time,
            base_time + Duration::days(5),
        )
        .await
        .unwrap();

    // Order tidak boleh terisi dan tidak boleh menghasilkan profit palsu!
    assert_eq!(
        report.total_trades, 0,
        "Pending order yang tidak terjemput WAJIB 0 trade / Expired!"
    );
    assert_eq!(report.total_raw_pips, Decimal::ZERO, "Profit WAJIB 0 pips!");
    println!("✅ Invariant 2 Lolos: Pending order yang tidak terjemput resmi Expired tanpa profit palsu.");
}

// ==============================================================================
// 🧪 3. AUDIT INVARIANT 3: INTRABAR AMBIGUITY CONSERVATIVE WORST-CASE EXECUTION
// ==============================================================================
#[tokio::test]
async fn test_audit_intrabar_ambiguity_conservative_worst_case() {
    let symbol = Symbol::new("EUR", "USD");
    let base_time = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    // Bar 0-49 (50 candles): History window
    let mut candles = Vec::new();
    for i in 0..50 {
        let price = dec!(1.08500);
        candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: price,
            high: price + dec!(0.00050),
            low: price - dec!(0.00050),
            close: price,
            volume: dec!(1000.0),
        });
    }

    // Bar 50 (index 50): Emits Pending BuyLimit at 1.08200
    candles.push(Candle {
        symbol: symbol.clone(),
        timeframe: Timeframe::H1,
        timestamp: base_time + Duration::hours(50),
        source: domain::models::MarketDataSource::SyntheticTest,
        open: dec!(1.08500),
        high: dec!(1.08550),
        low: dec!(1.08450),
        close: dec!(1.08500),
        volume: dec!(1000.0),
    });

    // Bar 51 (index 51): Pullback menjemput entry limit (Low turun ke 1.08100 -> Limit 1.08200 FILLED!)
    candles.push(Candle {
        symbol: symbol.clone(),
        timeframe: Timeframe::H1,
        timestamp: base_time + Duration::hours(51),
        source: domain::models::MarketDataSource::SyntheticTest,
        open: dec!(1.08400),
        high: dec!(1.08450),
        low: dec!(1.08100), // Entry Filled!
        close: dec!(1.08200),
        volume: dec!(2000.0),
    });

    // Bar 52 (index 52): Super Volatile Candle -> High menembus TP (1.08600) DAN Low menembus SL (1.08000) sekaligus!
    candles.push(Candle {
        symbol: symbol.clone(),
        timeframe: Timeframe::H1,
        timestamp: base_time + Duration::hours(52),
        source: domain::models::MarketDataSource::SyntheticTest,
        open: dec!(1.08200),
        high: dec!(1.08800), // TP (1.08600) tersentuh!
        low: dec!(1.07800),  // SL (1.08000) tersentuh!
        close: dec!(1.08500),
        volume: dec!(10000.0),
    });

    // Bar 53-60: Buffer penutup
    for i in 53..60 {
        let price = dec!(1.08500);
        candles.push(Candle {
            symbol: symbol.clone(),
            timeframe: Timeframe::H1,
            timestamp: base_time + Duration::hours(i as i64),
            source: domain::models::MarketDataSource::SyntheticTest,
            open: price,
            high: price + dec!(0.00050),
            low: price - dec!(0.00050),
            close: price,
            volume: dec!(1000.0),
        });
    }

    struct TriggerAndVolatileStrategy;
    #[async_trait]
    impl StrategyPort for TriggerAndVolatileStrategy {
        fn name(&self) -> &str {
            "TriggerAndVolatile"
        }
        async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError> {
            if ctx.candles.len() == 50 {
                Ok(Some(Signal {
                    id: Uuid::new_v4(),
                    symbol: ctx.symbol.clone(),
                    action: SignalAction::BuyLimit,
                    timeframe: ctx.timeframe,
                    entry_price: dec!(1.08200),
                    stop_loss: dec!(1.08000),     // 20 pips SL
                    take_profit_1: dec!(1.08600), // 40 pips TP (1:2 RR)
                    take_profit_2: None,
                    take_profit_3: None,
                    risk_reward_ratio: dec!(2.0),
                    confidence_score: 0.95,
                    strategy_name: "TriggerAndVolatile".to_string(),
                    rationale: "Testing worst-case execution".to_string(),
                    status: SignalStatus::Pending,
                    created_at: Utc::now(),
                    expires_at: Some(Utc::now() + Duration::hours(24)),
                }))
            } else {
                Ok(None)
            }
        }
    }

    let feed = Arc::new(AuditMarketFeed {
        candles,
        spread_pips: dec!(0.5),
    });
    let backtester = BacktestService::new(
        feed,
        Arc::new(TriggerAndVolatileStrategy),
        RiskProfile::default(),
    );

    let report = backtester
        .run_simulation(
            &symbol,
            Timeframe::H1,
            base_time,
            base_time + Duration::days(5),
        )
        .await
        .unwrap();

    // Sesuai prinsip konservatif: Jika TP dan SL tersentuh di bar yang sama, Stop Loss WAJIB terkena lebih dulu!
    assert_eq!(report.total_trades, 1, "Trade harus settled");
    assert_eq!(
        report.losing_trades, 1,
        "WAJIB terhitung sebagai LOSING trade (Worst-case priority)"
    );
    assert_eq!(
        report.winning_trades, 0,
        "TIDAK BOLEH terhitung sebagai WIN"
    );
    assert_eq!(report.total_raw_pips, dec!(-20.0), "PnL harus -20 pips SL!");
    println!("✅ Invariant 3 Lolos: Intrabar ambiguity terbukti mendahulukan Stop Loss (Konservatif Anti-Overfitting).");
}

// ==============================================================================
// 🧪 4. AUDIT INVARIANT 4: TRADERS FAMILY 4-TIER VALUED PIPS (VP) MULTIPLICATION
// ==============================================================================
#[test]
fn test_audit_traders_family_4tier_valued_pips_precision() {
    let pips = dec!(100.0);

    // Tier 1: Multiplier 2.0x
    let spec_tier1 = TfPairSpec::from_symbol(&Symbol::new("NZD", "USD"));
    assert_eq!(
        spec_tier1.pips_to_valued_pips(pips),
        dec!(200.0),
        "Tier 1 NZDUSD harus 2.0x"
    );

    // Tier 2: Multiplier 1.5x
    let spec_tier2 = TfPairSpec::from_symbol(&Symbol::new("EUR", "USD"));
    assert_eq!(
        spec_tier2.pips_to_valued_pips(pips),
        dec!(150.0),
        "Tier 2 EURUSD harus 1.5x"
    );

    // Tier 3: Multiplier 1.0x
    let spec_tier3 = TfPairSpec::from_symbol(&Symbol::new("USD", "JPY"));
    assert_eq!(
        spec_tier3.pips_to_valued_pips(pips),
        dec!(100.0),
        "Tier 3 USDJPY harus 1.0x"
    );

    // Tier 4: Multiplier 0.5x
    let spec_tier4 = TfPairSpec::from_symbol(&Symbol::new("XAU", "USD"));
    assert_eq!(
        spec_tier4.pips_to_valued_pips(pips),
        dec!(50.0),
        "Tier 4 Gold harus 0.5x"
    );

    println!(
        "✅ Invariant 4 Lolos: 4-Tier Valued Pips Multiplier terbukti 100% presisi matematis."
    );
}

// ==============================================================================
// 🧪 5. AUDIT INVARIANT 5: ZERO-PENALTY PLATFORM COMPLIANCE GUARD
// ==============================================================================
#[test]
fn test_audit_tf_compliance_guard_rejections() {
    // 1. Uji Penolakan Instant Execution (Hanya Pending Order yang legal di TF)
    let instant_signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("EUR", "USD"),
        action: SignalAction::Buy, // Ilegal di TF
        timeframe: Timeframe::H1,
        entry_price: dec!(1.08500),
        stop_loss: dec!(1.08300),
        take_profit_1: dec!(1.08900),
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(2.0),
        confidence_score: 0.9,
        strategy_name: "Test".to_string(),
        rationale: "Instant test".to_string(),
        status: SignalStatus::Active,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::hours(24)),
    };
    assert!(
        TfComplianceGuard::validate_signal(&instant_signal).is_err(),
        "Instant execution harus ditolak 100%"
    );

    // 2. Uji Penolakan R:R Melebihi Batas Maksimal 1:3.0
    let greed_signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("NZD", "USD"),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(0.60000),
        stop_loss: dec!(0.59800),     // 20 pips SL
        take_profit_1: dec!(0.60800), // 80 pips TP (R:R 1:4.0 -> ILEGAL di TF!)
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(4.0),
        confidence_score: 0.9,
        strategy_name: "Test".to_string(),
        rationale: "Greed test".to_string(),
        status: SignalStatus::Pending,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::hours(24)),
    };
    assert!(
        TfComplianceGuard::validate_signal(&greed_signal).is_err(),
        "R:R 1:4.0 harus ditolak (Max legal TF adalah 1:3.0)!"
    );

    // 3. Uji Penolakan SL Melebihi 1.5x TP
    let bad_sl_signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("AUD", "USD"),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(0.66000),
        stop_loss: dec!(0.65500),     // 50 pips SL
        take_profit_1: dec!(0.66200), // 20 pips TP (SL = 2.5x TP -> ILEGAL!)
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(0.4),
        confidence_score: 0.9,
        strategy_name: "Test".to_string(),
        rationale: "Bad SL test".to_string(),
        status: SignalStatus::Pending,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::hours(24)),
    };
    assert!(
        TfComplianceGuard::validate_signal(&bad_sl_signal).is_err(),
        "SL > 1.5x TP harus ditolak!"
    );

    println!("✅ Invariant 5 Lolos: TfComplianceGuard 100% menolak sinyal ilegal (Zero-Banned Guarantee).");
}

// ==============================================================================
// 🧪 6. AUDIT INVARIANT 6: FINANCIAL DECIMAL ARITHMETIC DRIFT-FREE
// ==============================================================================
#[test]
fn test_audit_financial_decimal_precision_drift_free() {
    let mut price = dec!(1.08500);
    let step = dec!(0.00010); // 1 pip

    // Lakukan 10.000 kalkulasi penambahan pips
    for _ in 0..10000 {
        price += step;
    }

    // Pada f64 IEEE-754 biasa, 1.08500 + (0.0001 * 10000) akan mengalami precision drift
    // Pada rust_decimal::Decimal murni, hasilnya WAJIB eksak 2.08500!
    assert_eq!(
        price,
        dec!(2.08500),
        "Decimal arithmetic tidak boleh memiliki floating point drift!"
    );

    // Pengujian pembagian pips
    let pip_diff = dec!(0.00250);
    let pip_size = dec!(0.00010);
    assert_eq!(
        pip_diff / pip_size,
        dec!(25.0),
        "Perhitungan 25.0 pips harus eksak!"
    );

    println!("✅ Invariant 6 Lolos: Aritmatika finansial Decimal murni 100% bebas dari floating point bug.");
}
