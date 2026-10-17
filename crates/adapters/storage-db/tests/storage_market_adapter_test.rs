use chrono::Utc;
use domain::models::{
    BrokerSymbolSpec, Candle, MarketDataSource, Symbol, Tick, Timeframe,
};
use domain::ports::{BrokerSpecPort, MarketDataPort, StoragePort, TickStoragePort};
use rust_decimal_macros::dec;
use std::sync::Arc;
use storage_db::{SqliteStorage, StorageMarketDataAdapter};

#[tokio::test]
async fn test_storage_market_adapter_full_bridge_lifecycle() {
    let storage = Arc::new(
        SqliteStorage::connect(":memory:")
            .await
            .expect("Koneksi SQLite memory harus sukses"),
    );

    let adapter = StorageMarketDataAdapter::with_tick_and_spec_storage(
        MarketDataSource::DukascopyEcn,
        storage.clone(),
        Some(storage.clone() as Arc<dyn TickStoragePort>),
        Some(storage.clone() as Arc<dyn BrokerSpecPort>),
    );

    let sym = Symbol::new("EUR", "USD");
    let now = Utc::now();

    // 1. Simpan spec ke storage dan verifikasi via adapter
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
        stop_level_pts: 25,
        freeze_level_pts: 0,
        min_lot: dec!(0.01),
        lot_step: dec!(0.01),
        max_lot: dec!(100.0),
        margin_required: dec!(1000.0),
    };
    storage.save_broker_spec(&spec).await.unwrap();

    let fetched_spec = adapter.get_broker_spec(&sym).await;
    assert!(fetched_spec.is_some());
    assert_eq!(fetched_spec.unwrap().stop_level_pts, 25);

    // 2. Simpan candle dan verifikasi via adapter
    let c1 = Candle {
        symbol: sym.clone(),
        timeframe: Timeframe::H1,
        timestamp: now,
        source: MarketDataSource::DukascopyEcn,
        open: dec!(1.0850),
        high: dec!(1.0880),
        low: dec!(1.0830),
        close: dec!(1.0860),
        volume: dec!(1250.0),
        spread: Some(dec!(0.00015)),
        real_volume: None,
    };
    storage.save_candles(&[c1]).await.unwrap();

    let recent = adapter.get_recent_candles(&sym, Timeframe::H1, 10).await.unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].close, dec!(1.0860));

    let range = adapter
        .get_historical_candles(
            &sym,
            Timeframe::H1,
            now - chrono::Duration::hours(1),
            now + chrono::Duration::hours(1),
        )
        .await
        .unwrap();
    assert_eq!(range.len(), 1);

    // 3. Simpan tick dan verifikasi via adapter
    let tick = Tick {
        symbol: sym.clone(),
        source: MarketDataSource::DukascopyEcn,
        bid: dec!(1.0860),
        ask: dec!(1.08615),
        timestamp: now,
        spread_pts: Some(15),
        volume: None,
    };
    storage.save_ticks(&[tick]).await.unwrap();

    let hist_ticks = adapter
        .get_historical_ticks(&sym, now - chrono::Duration::hours(1), now + chrono::Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(hist_ticks.len(), 1);
    assert_eq!(hist_ticks[0].bid, dec!(1.0860));

    let latest_tick = adapter.get_latest_tick(&sym).await.unwrap();
    assert_eq!(latest_tick.bid, dec!(1.0860));
}
