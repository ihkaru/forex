use chrono::Utc;
use domain::models::{
    BrokerSymbolSpec, Candle, MarketDataSource, Signal, SignalAction, SignalStatus, Symbol, Tick,
    Timeframe,
};
use domain::ports::{BrokerSpecPort, StoragePort, TickStoragePort};
use rust_decimal_macros::dec;
use storage_db::SqliteStorage;
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_storage_full_lifecycle() {
    let storage = SqliteStorage::connect(":memory:")
        .await
        .expect("Koneksi SQLite in-memory harus sukses");

    let sym = Symbol::new("EUR", "USD");

    // 1. Test Broker Spec Persistence
    let spec = BrokerSymbolSpec {
        symbol: sym.clone(),
        raw_ticker: "EURUSD".to_string(),
        digits: 5,
        point: dec!(0.00001),
        tick_value: dec!(1.0),
        tick_size: dec!(0.00001),
        contract_size: dec!(100000.0),
        swap_long: dec!(-0.7),
        swap_short: dec!(0.2),
        swap_rollover_3days: 3,
        stop_level_pts: 20,
        freeze_level_pts: 0,
        min_lot: dec!(0.01),
        lot_step: dec!(0.01),
        max_lot: dec!(100.0),
        margin_required: dec!(1000.0),
    };

    storage
        .save_broker_spec(&spec)
        .await
        .expect("Simpan broker spec harus sukses");

    let fetched_spec = storage
        .get_broker_spec(&sym)
        .await
        .expect("Get broker spec harus sukses")
        .expect("Spec harus ditemukan");

    assert_eq!(fetched_spec.raw_ticker, "EURUSD");
    assert_eq!(fetched_spec.stop_level_pts, 20);

    // 2. Test Batch Candle Persistence & Range Query
    let t0 = Utc::now();
    let c1 = Candle {
        symbol: sym.clone(),
        timeframe: Timeframe::H1,
        timestamp: t0,
        source: MarketDataSource::MrgRealMt4,
        open: dec!(1.0850),
        high: dec!(1.0890),
        low: dec!(1.0840),
        close: dec!(1.0880),
        volume: dec!(1500.0),
        spread: Some(dec!(0.00015)),
        real_volume: Some(dec!(100.0)),
    };
    let c2 = Candle {
        symbol: sym.clone(),
        timeframe: Timeframe::H1,
        timestamp: t0 + chrono::Duration::hours(1),
        source: MarketDataSource::MrgRealMt4,
        open: dec!(1.0880),
        high: dec!(1.0920),
        low: dec!(1.0870),
        close: dec!(1.0910),
        volume: dec!(2200.0),
        spread: Some(dec!(0.00018)),
        real_volume: Some(dec!(150.0)),
    };

    storage
        .save_candles(&[c1.clone(), c2.clone()])
        .await
        .expect("Simpan candles harus sukses");

    let candles = storage
        .get_candles(&sym, Timeframe::H1, 10)
        .await
        .expect("Get candles harus sukses");
    assert_eq!(candles.len(), 2);
    assert_eq!(candles[0].open, dec!(1.0850));
    assert_eq!(candles[1].close, dec!(1.0910));

    let range_candles = storage
        .get_historical_candles_range(
            &sym,
            Timeframe::H1,
            MarketDataSource::MrgRealMt4,
            t0,
            t0 + chrono::Duration::hours(1),
        )
        .await
        .expect("Range candles harus sukses");
    assert_eq!(range_candles.len(), 2);

    // 3. Test High Watermark
    let watermark = storage
        .get_high_watermark(&sym, Timeframe::H1, MarketDataSource::MrgRealMt4)
        .await
        .expect("Watermark harus sukses")
        .expect("Watermark harus ada");
    assert_eq!(watermark.timestamp(), (t0 + chrono::Duration::hours(1)).timestamp());

    // 4. Test Tick Storage & Range Query
    let tick1 = Tick {
        symbol: sym.clone(),
        timestamp: t0,
        source: MarketDataSource::MrgRealMt4,
        bid: dec!(1.0850),
        ask: dec!(1.0852),
        spread_pts: Some(20),
        volume: Some(dec!(5.0)),
    };
    storage
        .save_ticks(&[tick1])
        .await
        .expect("Simpan ticks harus sukses");

    let ticks = storage
        .get_ticks(&sym, MarketDataSource::MrgRealMt4, t0 - chrono::Duration::seconds(1), t0 + chrono::Duration::seconds(1))
        .await
        .expect("Get ticks harus sukses");
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].bid, dec!(1.0850));

    // 5. Test Signal Storage
    let sig = Signal {
        id: Uuid::new_v4(),
        symbol: sym.clone(),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(1.0850),
        stop_loss: dec!(1.0820),
        take_profit_1: dec!(1.0900),
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(1.67),
        confidence_score: 0.85,
        strategy_name: "PolaN_v8".to_string(),
        rationale: "Bullish Retracement at 61.8%".to_string(),
        status: SignalStatus::Active,
        created_at: t0,
        expires_at: Some(t0 + chrono::Duration::hours(24)),
    };
    storage.save_signal(&sig).await.expect("Simpan signal sukses");
    let fetched_sig = storage.get_signal(sig.id).await.expect("Get signal sukses").expect("Signal ada");
    assert_eq!(fetched_sig.strategy_name, "PolaN_v8");
}
