use broker_connector::{BrokerConnector, CtraderOpenApiConnector, Mt5SocketMessage};
use chrono::{Duration, Utc};
use domain::models::{Symbol, Timeframe};
use domain::ports::MarketDataPort;
use dukascopy_rs::DukascopyDownloader;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::test]
async fn test_hexagonal_polymorphism_and_cross_source_data_integrity() {
    let symbol = Symbol::new("EUR", "USD");

    // 1. Instansiasi 3 Sumber Data Berbeda (Adapters)
    let mt5_raw = Arc::new(BrokerConnector::new("MetaTrader5-Live"));
    let mt5_adapter: Arc<dyn MarketDataPort> = mt5_raw.clone();
    let ctrader_adapter: Arc<dyn MarketDataPort> =
        Arc::new(CtraderOpenApiConnector::new("app-id", "secret", false));
    let dukascopy_adapter: Arc<dyn MarketDataPort> = Arc::new(DukascopyDownloader::new());

    // 2. Verifikasi Fail-Fast: Adapter yang belum terhubung atau kosong DILARANG mengembalikan mock palsu!
    println!("🧪 Memverifikasi Protokol Fail-Fast Anti-Mock...");
    assert!(
        mt5_adapter.get_latest_tick(&symbol).await.is_err(),
        "BrokerConnector kosong WAJIB Fail-Fast (Err) saat buffer tick kosong"
    );
    assert!(
        mt5_adapter
            .get_recent_candles(&symbol, Timeframe::M15, 10)
            .await
            .is_err(),
        "BrokerConnector kosong WAJIB Fail-Fast (Err) saat buffer candle kosong"
    );
    assert!(
        ctrader_adapter.get_latest_tick(&symbol).await.is_err(),
        "cTrader offline WAJIB Fail-Fast (Err)"
    );
    assert!(
        dukascopy_adapter.get_latest_tick(&symbol).await.is_err(),
        "DukascopyDownloader batch WAJIB Fail-Fast (Err) untuk live tick"
    );
    println!("✅ Protokol Fail-Fast Terbukti 100% Menggagalkan Mock!");

    // 3. Ingest Data Riil dari MetaTrader 5 Socket Bridge
    let now = Utc::now();
    let tick_msg = Mt5SocketMessage::Tick {
        symbol: "EURUSD".to_string(),
        source: Some("MrgDemoMt4".to_string()),
        server: Some("MaxrichGroup-Demo".to_string()),
        bid: dec!(1.08500),
        ask: dec!(1.08515),
        spread_pts: 15,
        time_gmt: now.timestamp(),
    };
    mt5_raw
        .ingest_socket_message(tick_msg)
        .await
        .expect("Ingest tick riil harus sukses");

    for i in 0..10 {
        let bar_time = now - Duration::minutes((10 - i) * 15);
        let bar_msg = Mt5SocketMessage::Bar {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            timeframe: "M15".to_string(),
            open: dec!(1.08450) + dec!(0.00005) * Decimal::from(i),
            high: dec!(1.08550) + dec!(0.00005) * Decimal::from(i),
            low: dec!(1.08400) + dec!(0.00005) * Decimal::from(i),
            close: dec!(1.08510) + dec!(0.00005) * Decimal::from(i),
            volume: dec!(500) + dec!(10) * Decimal::from(i),
            time_gmt: bar_time.timestamp(),
        };
        mt5_raw
            .ingest_socket_message(bar_msg)
            .await
            .expect("Ingest bar riil harus sukses");
    }

    // 4. Validasi Integritas Data Riil MT5 Adapter
    let tick = mt5_adapter
        .get_latest_tick(&symbol)
        .await
        .expect("Harus berhasil mengambil tick setelah di-ingest");

    assert!(tick.bid > Decimal::ZERO, "Bid price harus > 0");
    assert!(tick.ask > Decimal::ZERO, "Ask price harus > 0");
    assert!(
        tick.ask >= tick.bid,
        "Ask ({}) tidak boleh lebih kecil dari Bid ({})",
        tick.ask,
        tick.bid
    );
    assert_eq!(tick.timestamp.timezone(), chrono::Utc);

    let candles = mt5_adapter
        .get_recent_candles(&symbol, Timeframe::M15, 10)
        .await
        .expect("Harus berhasil mengambil candles");
    assert_eq!(candles.len(), 10);

    for candle in candles {
        assert!(candle.high >= candle.low, "High harus >= Low");
        assert!(candle.high >= candle.open, "High harus >= Open");
        assert!(candle.high >= candle.close, "High harus >= Close");
        assert!(candle.low <= candle.open, "Low harus <= Open");
        assert!(candle.low <= candle.close, "Low harus <= Close");
        assert!(candle.volume >= Decimal::ZERO, "Volume harus >= 0");
    }

    println!("✅ Integritas Data Riil MT5 Lolos 100%!");
}
