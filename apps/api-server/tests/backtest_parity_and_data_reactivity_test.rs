use api_server::create_router;
use api_server::state::{AppState, RealHistoricalMarketAdapter};
use application::services::BacktestService;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{TimeZone, Utc};
use domain::models::{
    Candle, PolaNStrategy, RiskProfile, Symbol, Timeframe,
};
use domain::ports::StoragePort;
use http_body_util::BodyExt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

/// Parser desimal tahan banting (mendukung string, float, dan integer dari JSON)
fn parse_dec(v: &Value) -> Decimal {
    if let Some(s) = v.as_str() {
        s.parse().unwrap()
    } else if let Some(f) = v.as_f64() {
        Decimal::from_f64_retain(f).unwrap()
    } else if let Some(i) = v.as_i64() {
        Decimal::from(i)
    } else {
        panic!("Invalid decimal JSON value: {:?}", v);
    }
}

/// Helper pembanding desimal finansial tahan toleransi floating-point JSON serialization
fn assert_dec_eq(left: Decimal, right: Decimal, msg: &str) {
    let diff = (left - right).abs();
    assert!(
        diff < dec!(0.0001),
        "{}: left ({}) != right ({}) (diff: {})",
        msg, left, right, diff
    );
}

/// Inisialisasi state pengujian berbasis pipeline persisten StorageMarketDataAdapter
async fn build_test_state() -> Arc<AppState> {
    let storage = Arc::new(storage_db::InMemoryStorage::new());
    let ingestion_service = Arc::new(application::services::MarketIngestionService::new(
        storage.clone(),
    ));

    // Seeding data historis nyata EURGBP dan EURUSD dari disk ke storage
    for p in &["EURGBP", "EURUSD"] {
        if let Some(sym) = Symbol::from_symbol_str(p) {
            let _ = ingestion_service.seed_storage_from_disk(&sym).await;
        }
    }

    let market_adapter: Arc<dyn domain::ports::MarketDataPort> = Arc::new(
        storage_db::StorageMarketDataAdapter::with_tick_and_spec_storage(
            domain::models::MarketDataSource::DukascopyEcn,
            storage.clone(),
            Some(storage.clone()),
            Some(storage.clone()),
        ),
    );
    let broker_connector = Arc::new(broker_connector::BrokerConnector::new("TestBroker"));
    let mut router = application::services::MarketDataRouterService::new();
    router.register(market_adapter.clone());
    router.register(broker_connector.clone());
    let router = Arc::new(router);

    // Gunakan strategi kuantitatif resmi terstandarisasi (V8 Titan Pro)
    let strategy = Arc::new(PolaNStrategy::v8_titan_pro());
    Arc::new(AppState {
        market_adapter,
        broker_connector,
        router,
        strategy,
        storage: storage.clone(),
        tick_storage: storage.clone(),
        broker_spec_storage: storage,
        ingestion_service,
    })
}

/// 1. TES DETERMINISME & PARITAS LINTAS CHANNEL (Cross-Surface Invariant Parity)
/// Memastikan hasil backtest pada data yang sama menghasilkan metrik yang 100% IDENTIK
/// baik dipanggil via Direct Service Layer maupun via REST API yang ditampilkan di UI.
#[tokio::test]
async fn test_backtest_cross_surface_determinism_parity() {
    let state = build_test_state().await;
    let app = create_router(state.clone());

    let sym = Symbol::from_symbol_str("EURGBP").expect("Valid symbol");
    let from_dt = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    let to_dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    // Jalur A: Evaluasi Langsung via Application Service Layer (Direct BacktestService)
    let direct_service = BacktestService::new(
        state.market_adapter.clone(),
        state.strategy.clone(),
        RiskProfile::default(),
    );
    let direct_detailed = direct_service
        .run_simulation_detailed(&sym, Timeframe::H1, from_dt, to_dt)
        .await
        .expect("Direct backtest simulation harus sukses");

    // Jalur B: Evaluasi via REST API Handler UI (/api/backtest/detailed/:symbol)
    let req = Request::builder()
        .uri(format!(
            "/api/backtest/detailed/EURGBP?from={}&to={}",
            from_dt.timestamp(),
            to_dt.timestamp()
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let api_detailed: Value = serde_json::from_slice(&bytes).unwrap();

    // Verifikasi 1: Paritas Metrik Portfolio & Laporan Ringkas
    let api_report = &api_detailed["report"];
    assert_eq!(
        api_report["total_trades"].as_u64().unwrap() as usize,
        direct_detailed.report.total_trades,
        "Total trades via API dan Direct Service WAJIB sama persis"
    );
    assert_eq!(
        api_report["winning_trades"].as_u64().unwrap() as usize,
        direct_detailed.report.winning_trades,
        "Winning trades WAJIB sama persis"
    );
    assert_eq!(
        api_report["losing_trades"].as_u64().unwrap() as usize,
        direct_detailed.report.losing_trades,
        "Losing trades WAJIB sama persis"
    );
    assert_dec_eq(
        parse_dec(&api_report["total_raw_pips"]),
        direct_detailed.report.total_raw_pips,
        "Total Raw Pips WAJIB identik tanpa drift",
    );
    assert_dec_eq(
        parse_dec(&api_report["total_valued_pips"]),
        direct_detailed.report.total_valued_pips,
        "Total Valued Pips WAJIB identik tanpa drift",
    );
    assert_dec_eq(
        parse_dec(&api_report["max_drawdown_pips"]),
        direct_detailed.report.max_drawdown_pips,
        "Max Drawdown Pips WAJIB identik",
    );

    // Verifikasi 2: Paritas Rinci Setiap Trade Eksekusi (Trade-by-Trade Inspection)
    let api_trades = api_detailed["trades"].as_array().expect("Array trades");
    assert_eq!(
        api_trades.len(),
        direct_detailed.trades.len(),
        "Jumlah list trades harus sama persis"
    );

    for (i, t_direct) in direct_detailed.trades.iter().enumerate() {
        let t_api = &api_trades[i];
        assert_eq!(
            t_api["open_time"].as_i64().unwrap(),
            t_direct.open_time.timestamp(),
            "Trade #{} open time harus identik",
            i
        );
        assert_dec_eq(
            parse_dec(&t_api["open_price"]),
            t_direct.open_price,
            "Trade open price harus identik",
        );
        assert_dec_eq(
            parse_dec(&t_api["pnl_pips"]),
            t_direct.realized_pnl.unwrap(),
            "Trade pnl pips harus identik",
        );
    }
}

/// 2. TES SENSITIVITAS DATA (Dynamic Reactivity When Data Changes)
/// Memastikan bahwa jika data diganti / diubah rentangnya, maka hasil backtest
/// secara dinamis IKUT BERUBAH (tidak hardcoded atau ter-cache statis).
#[tokio::test]
async fn test_backtest_results_change_dynamically_when_data_changes() {
    let state = build_test_state().await;
    let app = create_router(state.clone());

    // Periode 1: Semester 1 (2023-01-01 s/d 2023-07-01)
    let t1_from = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap().timestamp();
    let t1_to = Utc.with_ymd_and_hms(2023, 7, 1, 0, 0, 0).unwrap().timestamp();

    let req1 = Request::builder()
        .uri(format!("/api/backtest/detailed/EURUSD?from={}&to={}", t1_from, t1_to))
        .body(Body::empty())
        .unwrap();
    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let bytes1 = resp1.into_body().collect().await.unwrap().to_bytes();
    let res1: Value = serde_json::from_slice(&bytes1).unwrap();

    // Periode 2: 2 Tahun Penuh (2023-01-01 s/d 2025-01-01)
    let t2_from = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap().timestamp();
    let t2_to = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap().timestamp();

    let req2 = Request::builder()
        .uri(format!("/api/backtest/detailed/EURUSD?from={}&to={}", t2_from, t2_to))
        .body(Body::empty())
        .unwrap();
    let resp2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let bytes2 = resp2.into_body().collect().await.unwrap().to_bytes();
    let res2: Value = serde_json::from_slice(&bytes2).unwrap();

    let trades1 = res1["report"]["total_trades"].as_u64().unwrap();
    let trades2 = res2["report"]["total_trades"].as_u64().unwrap();
    let vp1 = parse_dec(&res1["report"]["total_valued_pips"]);
    let vp2 = parse_dec(&res2["report"]["total_valued_pips"]);

    // Pembuktian Empiris: Data berbeda WAJIB menghasilkan hasil berbeda
    assert_ne!(
        trades1, trades2,
        "Jumlah trade pada data 6 bulan vs 2 tahun harus berbeda! (trades1: {}, trades2: {})",
        trades1, trades2
    );
    assert_ne!(
        vp1, vp2,
        "Total Valued Pips pada data berbeda harus berbeda! (vp1: {}, vp2: {})",
        vp1, vp2
    );

    // Dan untuk Periode 2, uji paritas determinisme antara Direct Service dan API UI
    let sym = Symbol::from_symbol_str("EURUSD").unwrap();
    let direct_service = BacktestService::new(
        state.market_adapter.clone(),
        state.strategy.clone(),
        RiskProfile::default(),
    );
    let from_dt = Utc.timestamp_opt(t2_from, 0).single().unwrap();
    let to_dt = Utc.timestamp_opt(t2_to, 0).single().unwrap();
    let direct_res2 = direct_service
        .run_simulation_detailed(&sym, Timeframe::H1, from_dt, to_dt)
        .await
        .unwrap();

    assert_eq!(trades2 as usize, direct_res2.report.total_trades);
    assert_dec_eq(
        vp2,
        direct_res2.report.total_valued_pips,
        "Valued pips WAJIB identik",
    );
}

/// 3. TES REAKTIVITAS MUTASI PERSISTENSI STORAGE (Live Persistence Mutation Reactivity)
/// Membuktikan bahwa saat lilin baru ditambahkan ke storage, hasil backtest langsung
/// merefleksikan lilin baru tersebut tanpa perlu restart server.
#[tokio::test]
async fn test_backtest_reacts_dynamically_to_storage_mutations() {
    let storage = Arc::new(storage_db::InMemoryStorage::new());
    let ingestion_service = Arc::new(application::services::MarketIngestionService::new(
        storage.clone(),
    ));

    let sym = Symbol::from_symbol_str("EURGBP").unwrap();
    let all_candles = RealHistoricalMarketAdapter::load_real_market_candles(&sym).unwrap();

    // Pisahkan dataset nyata menjadi 2 batch:
    // Batch 1: Lilin tahun 2023
    let t_start_2023 = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    let t_end_2023 = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
    let t_end_2024 = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

    let candles_2023: Vec<Candle> = all_candles
        .iter()
        .filter(|c| c.timestamp >= t_start_2023 && c.timestamp <= t_end_2023)
        .cloned()
        .collect();

    let candles_2024: Vec<Candle> = all_candles
        .iter()
        .filter(|c| c.timestamp > t_end_2023 && c.timestamp <= t_end_2024)
        .cloned()
        .collect();

    assert!(!candles_2023.is_empty(), "Dataset 2023 harus ada");
    assert!(!candles_2024.is_empty(), "Dataset 2024 harus ada");

    // Simpan Batch 1 (Hanya tahun 2023)
    storage.save_candles(&candles_2023).await.unwrap();

    let market_adapter: Arc<dyn domain::ports::MarketDataPort> = Arc::new(
        storage_db::StorageMarketDataAdapter::with_tick_and_spec_storage(
            domain::models::MarketDataSource::DukascopyEcn,
            storage.clone(),
            Some(storage.clone()),
            Some(storage.clone()),
        ),
    );
    let broker_connector = Arc::new(broker_connector::BrokerConnector::new("TestBroker"));
    let mut router = application::services::MarketDataRouterService::new();
    router.register(market_adapter.clone());
    let router = Arc::new(router);
    let strategy = Arc::new(PolaNStrategy::v8_titan_pro());

    let state = Arc::new(AppState {
        market_adapter,
        broker_connector,
        router,
        strategy,
        storage: storage.clone(),
        tick_storage: storage.clone(),
        broker_spec_storage: storage.clone(),
        ingestion_service,
    });
    let app = create_router(state.clone());

    // Evaluasi Tahap 1: Backtest hanya pada data 2023
    let req1 = Request::builder()
        .uri(format!(
            "/api/backtest/detailed/EURGBP?from={}&to={}",
            t_start_2023.timestamp(),
            t_end_2024.timestamp()
        ))
        .body(Body::empty())
        .unwrap();
    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let bytes1 = resp1.into_body().collect().await.unwrap().to_bytes();
    let data1: Value = serde_json::from_slice(&bytes1).unwrap();
    let trades_phase1 = data1["report"]["total_trades"].as_u64().unwrap();

    // Sekarang lakukan MUTASI DATA PERSISTENSI: Injeksi lilin tahun 2024 ke storage
    storage.save_candles(&candles_2024).await.unwrap();

    // Evaluasi Tahap 2: Panggil kembali API endpoint tanpa restart server
    let req2 = Request::builder()
        .uri(format!(
            "/api/backtest/detailed/EURGBP?from={}&to={}",
            t_start_2023.timestamp(),
            t_end_2024.timestamp()
        ))
        .body(Body::empty())
        .unwrap();
    let resp2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let bytes2 = resp2.into_body().collect().await.unwrap().to_bytes();
    let data2: Value = serde_json::from_slice(&bytes2).unwrap();

    let trades_phase2 = data2["report"]["total_trades"].as_u64().unwrap();
    let vp_phase2 = parse_dec(&data2["report"]["total_valued_pips"]);

    // Bukti Mutasi Dinamis: Hasil harus langsung bertambah karena data baru masuk
    assert!(
        trades_phase2 > trades_phase1,
        "Jumlah trade setelah injeksi data baru harus bertambah! (Phase 1: {}, Phase 2: {})",
        trades_phase1,
        trades_phase2
    );

    // Paritas UI & Direct Service pada data hasil mutasi
    let direct_service = BacktestService::new(
        state.market_adapter.clone(),
        state.strategy.clone(),
        RiskProfile::default(),
    );
    let direct_res2 = direct_service
        .run_simulation_detailed(&sym, Timeframe::H1, t_start_2023, t_end_2024)
        .await
        .unwrap();

    assert_eq!(
        trades_phase2 as usize,
        direct_res2.report.total_trades,
        "Metrik API setelah mutasi storage WAJIB 100% sama dengan Direct Service"
    );
    assert_dec_eq(
        vp_phase2,
        direct_res2.report.total_valued_pips,
        "Valued pips setelah mutasi storage WAJIB 100% identik",
    );
}
