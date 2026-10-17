use domain::models::{Symbol, Timeframe};
use domain::ports::MarketDataPort;
use rust_decimal_macros::dec;

use crate::connector::BrokerConnector;
use crate::messages::{Mt5SocketMessage, MAX_CANDLE_CAPACITY};
use crate::validator::DataIntegrityValidator;

#[test]
fn test_negative_spread_integrity_rejection() {
    let bid = dec!(1.08500);
    let ask = dec!(1.08490); // Ask < Bid (Rusak)

    let result = DataIntegrityValidator::validate_tick(bid, ask);
    assert!(result.is_err());
}

#[test]
fn test_valid_spread_acceptance() {
    let bid = dec!(1.08500);
    let ask = dec!(1.08515); // Normal spread 1.5 pips

    let result = DataIntegrityValidator::validate_tick(bid, ask);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ingest_socket_tick_message() {
    let connector = BrokerConnector::new("MetaTrader5-Socket");
    let msg = Mt5SocketMessage::Tick {
        symbol: "EURUSD".to_string(),
        source: Some("MrgDemoMt4".to_string()),
        server: Some("MaxrichGroup-Demo".to_string()),
        bid: dec!(1.08500),
        ask: dec!(1.08515),
        spread_pts: 15,
        time_gmt: 1720000000,
    };

    connector.ingest_socket_message(msg).await.unwrap();

    let symbol = Symbol::new("EUR", "USD");
    let tick = connector.get_latest_tick(&symbol).await.unwrap();
    assert_eq!(tick.bid, dec!(1.08500));
    assert_eq!(tick.ask, dec!(1.08515));
}

#[tokio::test]
async fn test_monotonic_tick_rejection() {
    let connector = BrokerConnector::new("MetaTrader5-Socket");
    let symbol = Symbol::new("EUR", "USD");

    // Tick awal pada t = 1000
    connector
        .ingest_socket_message(Mt5SocketMessage::Tick {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            server: None,
            bid: dec!(1.08500),
            ask: dec!(1.08515),
            spread_pts: 15,
            time_gmt: 1000,
        })
        .await
        .unwrap();

    // Tick baru pada t = 1010
    connector
        .ingest_socket_message(Mt5SocketMessage::Tick {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            server: None,
            bid: dec!(1.08550),
            ask: dec!(1.08565),
            spread_pts: 15,
            time_gmt: 1010,
        })
        .await
        .unwrap();

    let tick = connector.get_latest_tick(&symbol).await.unwrap();
    assert_eq!(tick.bid, dec!(1.08550));

    // Out-of-order tick pada t = 1005 (lebih usang dari 1010, akibat TCP jitter)
    connector
        .ingest_socket_message(Mt5SocketMessage::Tick {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            server: None,
            bid: dec!(1.08400),
            ask: dec!(1.08415),
            spread_pts: 15,
            time_gmt: 1005,
        })
        .await
        .unwrap();

    // State harus tetap dipertahankan pada data terkini (t = 1010)
    let tick_after = connector.get_latest_tick(&symbol).await.unwrap();
    assert_eq!(tick_after.bid, dec!(1.08550));
}

#[tokio::test]
async fn test_candle_buffer_upsert_and_bounded_capacity() {
    let connector = BrokerConnector::new("MetaTrader5-Socket");
    let symbol = Symbol::new("EUR", "USD");

    // 1. Ingest bar pertama t = 3600
    connector
        .ingest_socket_message(Mt5SocketMessage::Bar {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            timeframe: "H1".to_string(),
            open: dec!(1.08000),
            high: dec!(1.08200),
            low: dec!(1.07900),
            close: dec!(1.08100),
            volume: dec!(100),
            time_gmt: 3600,
        })
        .await
        .unwrap();

    let candles = connector
        .get_recent_candles(&symbol, Timeframe::H1, 10)
        .await
        .unwrap();
    assert_eq!(candles.len(), 1);
    assert_eq!(candles[0].close, dec!(1.08100));

    // 2. Ingest update bar berjalan di timestamp yang sama t = 3600 (forming bar tick update)
    connector
        .ingest_socket_message(Mt5SocketMessage::Bar {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            timeframe: "H1".to_string(),
            open: dec!(1.08000),
            high: dec!(1.08300), // new high
            low: dec!(1.07900),
            close: dec!(1.08250), // new close
            volume: dec!(150),
            time_gmt: 3600,
        })
        .await
        .unwrap();

    let candles_updated = connector
        .get_recent_candles(&symbol, Timeframe::H1, 10)
        .await
        .unwrap();
    assert_eq!(
        candles_updated.len(),
        1,
        "Idempotent upsert harus mencegah duplikasi bar di timestamp yang sama"
    );
    assert_eq!(candles_updated[0].high, dec!(1.08300));
    assert_eq!(candles_updated[0].close, dec!(1.08250));

    // 3. Uji bounded capacity: Kirim 550 bar berurutan
    for i in 1..=550 {
        connector
            .ingest_socket_message(Mt5SocketMessage::Bar {
            symbol: "EURUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            timeframe: "H1".to_string(),
            open: dec!(1.08000),
            high: dec!(1.08300),
            low: dec!(1.07900),
            close: dec!(1.08250),
            volume: dec!(100),
            time_gmt: 3600 + (i * 3600),
        })
        .await
        .unwrap();
    }

    let all_stored = connector
        .get_recent_candles(&symbol, Timeframe::H1, 1000)
        .await
        .unwrap();
    assert_eq!(
        all_stored.len(),
        MAX_CANDLE_CAPACITY,
        "Circular buffer harus dibatasi tepat di MAX_CANDLE_CAPACITY ({})",
        MAX_CANDLE_CAPACITY
    );
}

#[tokio::test]
async fn test_raw_symbol_preservation() {
    let connector = BrokerConnector::new("MetaTrader4-Socket");
    let msg = Mt5SocketMessage::Bar {
        symbol: "XAUUSD_MRG".to_string(),
        source: Some("MrgRealMt4".to_string()),
        timeframe: "H1".to_string(),
        open: dec!(2750.00),
        high: dec!(2755.00),
        low: dec!(2748.00),
        close: dec!(2752.50),
        volume: dec!(500),
        time_gmt: 1720000000,
    };

    connector.ingest_socket_message(msg).await.unwrap();

    let symbol = Symbol::from_symbol_str("XAUUSD_MRG").unwrap();
    assert_eq!(symbol.base, "XAU");
    assert_eq!(symbol.quote, "USD");
    assert_eq!(symbol.raw_ticker(), "XAUUSD_MRG");

    let raw = connector.get_raw_symbol(&symbol).await;
    assert_eq!(raw, Some("XAUUSD_MRG".to_string()));
}

#[tokio::test]
async fn test_clear_all_data() {
    let connector = BrokerConnector::new("MetaTrader4-Socket");
    let msg = Mt5SocketMessage::Bar {
        symbol: "EURUSD".to_string(),
        source: Some("MrgRealMt4".to_string()),
        timeframe: "H1".to_string(),
        open: dec!(1.08000),
        high: dec!(1.08300),
        low: dec!(1.07900),
        close: dec!(1.08250),
        volume: dec!(100),
        time_gmt: 1720000000,
    };

    connector.ingest_socket_message(msg).await.unwrap();
    let symbol = Symbol::new("EUR", "USD");
    let candles = connector.get_recent_candles(&symbol, Timeframe::H1, 10).await.unwrap();
    assert_eq!(candles.len(), 1);

    let cleared = connector.clear_all_data().await;
    assert_eq!(cleared, 1);

    let query_after = connector.get_recent_candles(&symbol, Timeframe::H1, 10).await;
    assert!(query_after.is_err(), "Setelah clear_all_data, buffer harus kosong");
}
