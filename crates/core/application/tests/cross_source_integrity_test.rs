use broker_connector::{BrokerConnector, CtraderOpenApiConnector};
use domain::models::{Symbol, Timeframe};
use domain::ports::MarketDataPort;
use dukascopy_rs::DukascopyDownloader;
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_hexagonal_polymorphism_and_cross_source_data_integrity() {
    let symbol = Symbol::new("EUR", "USD");

    // 1. Instansiasi 3 Sumber Data Berbeda (Adapters)
    let mt5_adapter: Arc<dyn MarketDataPort> = Arc::new(BrokerConnector::new("MetaTrader5-Live"));
    let ctrader_adapter: Arc<dyn MarketDataPort> = Arc::new(CtraderOpenApiConnector::new("app-id", "secret", false));
    let dukascopy_adapter: Arc<dyn MarketDataPort> = Arc::new(DukascopyDownloader::new());

    let sources: Vec<(&str, Arc<dyn MarketDataPort>)> = vec![
        ("MetaTrader 5 EA Bridge", mt5_adapter),
        ("cTrader Open API", ctrader_adapter),
        ("Dukascopy Swiss Bank Feed", dukascopy_adapter),
    ];

    // 2. Pengujian Integritas Data Lintas Sumber (Cross-Source Invariant Check)
    for (source_name, source) in sources {
        println!("🔍 Menguji Integritas Data dari Sumber: {}", source_name);

        // A. Validasi Integritas Tick
        let tick = source.get_latest_tick(&symbol).await.expect("Harus berhasil mengambil tick");
        
        // Invariant 1: Harga harus bernilai positif (> 0)
        assert!(tick.bid > Decimal::ZERO, "{} Bid price harus > 0", source_name);
        assert!(tick.ask > Decimal::ZERO, "{} Ask price harus > 0", source_name);

        // Invariant 2: Ask tidak boleh lebih kecil dari Bid (Anti Negative Spread)
        assert!(tick.ask >= tick.bid, "{} Ask ({}) tidak boleh lebih kecil dari Bid ({})", source_name, tick.ask, tick.bid);

        // Invariant 3: Timestamp ter-normalisasi ke UTC
        assert_eq!(tick.timestamp.timezone(), chrono::Utc, "{} Timezone harus UTC", source_name);

        // B. Validasi Integritas Candlestick (OHLCV)
        let candles = source.get_recent_candles(&symbol, Timeframe::M15, 10).await.expect("Harus berhasil mengambil candles");
        assert!(!candles.is_empty(), "{} Candles buffer tidak boleh kosong", source_name);

        for candle in candles {
            // Invariant 4: High adalah harga tertinggi pada bar
            assert!(candle.high >= candle.low, "{} High harus >= Low", source_name);
            assert!(candle.high >= candle.open, "{} High harus >= Open", source_name);
            assert!(candle.high >= candle.close, "{} High harus >= Close", source_name);

            // Invariant 5: Low adalah harga terendah pada bar
            assert!(candle.low <= candle.open, "{} Low harus <= Open", source_name);
            assert!(candle.low <= candle.close, "{} Low harus <= Close", source_name);

            // Invariant 6: Volume bernilai positif atau nol
            assert!(candle.volume >= Decimal::ZERO, "{} Volume harus >= 0", source_name);
        }

        println!("✅ Integritas Data Lolos 100% untuk: {}", source_name);
    }
}
