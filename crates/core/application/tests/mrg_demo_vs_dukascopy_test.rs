use chrono::{Duration, TimeZone, Utc};
use domain::models::{CandleQuery, MarketDataSource, Symbol, Timeframe};
use domain::ports::MarketDataPort;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::test]
async fn test_mrg_demo_candles_strictly_isolated_from_dukascopy() {
    let broker_connector = Arc::new(broker_connector::BrokerConnector::new("MRG_Demo_Bridge"));
    let symbol = Symbol::new("XAU", "USD");

    // 1. Simulasikan pengiriman 100 candle dari MRG Demo (mulai 23 Juni 2026)
    let june_23_2026 = Utc.with_ymd_and_hms(2026, 6, 23, 0, 0, 0).unwrap();
    for i in 0..100 {
        let bar_time = june_23_2026 + Duration::hours(i);
        let msg = broker_connector::Mt5SocketMessage::Bar {
            symbol: "XAUUSD".to_string(),
            source: Some("MrgDemoMt4".to_string()),
            timeframe: "H1".to_string(),
            open: dec!(4600.0) + dec!(0.5) * rust_decimal::Decimal::from(i),
            high: dec!(4605.0) + dec!(0.5) * rust_decimal::Decimal::from(i),
            low: dec!(4595.0) + dec!(0.5) * rust_decimal::Decimal::from(i),
            close: dec!(4602.0) + dec!(0.5) * rust_decimal::Decimal::from(i),
            volume: dec!(100.0),
            time_gmt: bar_time.timestamp(),
        };
        broker_connector.ingest_socket_message(msg).await.unwrap();
    }

    // 2. Query sumber MRG Demo
    let mrg_query = CandleQuery {
        symbol: symbol.clone(),
        timeframe: Timeframe::H1,
        source: MarketDataSource::MrgDemoMt4,
        from: None,
        to: None,
        limit: Some(500),
    };

    let mrg_candles = broker_connector.query_candles(&mrg_query).await.unwrap();

    // Verifikasi: Semua candle MRG Demo WAJIB berumur >= 23 Juni 2026!
    assert_eq!(
        mrg_candles.len(),
        100,
        "MRG Demo harus memiliki tepat 100 candle"
    );
    for candle in &mrg_candles {
        assert!(
            candle.timestamp >= june_23_2026,
            "Candle MRG Demo tidak boleh ada yang sebelum 23 Juni 2026! Ditemukan: {}",
            candle.timestamp
        );
        assert_eq!(
            candle.source,
            MarketDataSource::MrgDemoMt4,
            "Provenance tag harus MrgDemoMt4"
        );
    }

    // Candle pertama MRG Demo harus tepat 23 Juni 2026 00:00:00 UTC
    assert_eq!(mrg_candles.first().unwrap().timestamp, june_23_2026);
}
