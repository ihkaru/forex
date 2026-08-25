use api_server::create_router;
use api_server::state::{AppState, RealHistoricalMarketAdapter};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use domain::models::PolaNStrategy;
use http_body_util::BodyExt;
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

fn build_test_state() -> Arc<AppState> {
    let market_adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let broker_connector = Arc::new(broker_connector::BrokerConnector::new("TestBroker"));
    let mut router = application::services::MarketDataRouterService::new();
    router.register(market_adapter.clone());
    router.register(broker_connector.clone());
    let router = Arc::new(router);

    let strategy = Arc::new(PolaNStrategy::default());
    let storage = Arc::new(storage_db::InMemoryStorage::new());
    let ingestion_service = Arc::new(application::services::MarketIngestionService::new(
        storage.clone(),
    ));
    Arc::new(AppState {
        market_adapter,
        broker_connector,
        router,
        strategy,
        storage,
        ingestion_service,
    })
}

#[tokio::test]
async fn test_market_candles_api_rejects_request_without_source() {
    let state = build_test_state();
    let app = create_router(state);

    let req = Request::builder()
        .uri("/api/market/candles/EURUSD")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "Request candle tanpa source harus ditolak demi Data Integrity"
    );
}

#[tokio::test]
async fn test_market_candles_api_returns_real_historical_data() {
    let state = build_test_state();
    let app = create_router(state);

    let pairs = ["EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD"];

    for pair in pairs {
        let req = Request::builder()
            .uri(format!(
                "/api/market/candles/{}?source=dukascopy&limit=15000",
                pair
            ))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "Gagal pada pair {}", pair);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let candles: Vec<Value> = serde_json::from_slice(&body_bytes).unwrap();

        // 1. Pastikan bukan array kosong dan memiliki data historis nyata (> 10.000 bar)
        assert!(
            candles.len() > 10000,
            "Pair {} memiliki {} bar, harusnya > 10000 bar data nyata",
            pair,
            candles.len()
        );

        // 2. Verifikasi Invariant Matematika Lilin pada setiap bar nyata
        let mut prev_time = 0i64;
        for c in &candles {
            let time = c["time"].as_i64().expect("Missing time field");
            assert_eq!(c["source"].as_str(), Some("DukascopyEcn"));
            assert!(
                time > prev_time,
                "Timestamp lilin harus strictly monotonic: {} <= {}",
                time,
                prev_time
            );
            prev_time = time;

            let parse_dec = |v: &Value| -> Decimal {
                if let Some(s) = v.as_str() {
                    s.parse().unwrap()
                } else if let Some(f) = v.as_f64() {
                    Decimal::from_f64_retain(f).unwrap()
                } else {
                    panic!("Invalid decimal value: {:?}", v);
                }
            };

            let open = parse_dec(&c["open"]);
            let high = parse_dec(&c["high"]);
            let low = parse_dec(&c["low"]);
            let close = parse_dec(&c["close"]);
            let volume = parse_dec(&c["volume"]);

            assert!(high >= open, "High ({}) harus >= Open ({})", high, open);
            assert!(high >= close, "High ({}) harus >= Close ({})", high, close);
            assert!(low <= open, "Low ({}) harus <= Open ({})", low, open);
            assert!(low <= close, "Low ({}) harus <= Close ({})", low, close);
            assert!(volume >= Decimal::ZERO, "Volume harus >= 0");
        }
    }
}

#[tokio::test]
async fn test_api_server_health_and_scorecard_endpoints() {
    let state = build_test_state();
    let app = create_router(state);

    // Health
    let req = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let health: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(health["status"], "ONLINE");
    assert!(health["total_candles_loaded"].as_u64().unwrap() > 60000);

    // Scorecard
    let req = Request::builder()
        .uri("/api/scorecard")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let scorecard: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(scorecard["total_score"].as_u64().unwrap() <= 28);
    assert_eq!(scorecard["pillars"].as_array().unwrap().len(), 7);
}

#[tokio::test]
async fn test_api_server_monte_carlo_endpoint_guarantees_200_ok() {
    let state = build_test_state();
    let app = create_router(state);

    let req = Request::builder()
        .uri("/api/monte-carlo/EURGBP")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let mc: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(mc["symbol"], "EURGBP");
    assert_eq!(mc["iterations"], 1000);
    assert!(!mc["equity_paths"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_api_server_detailed_backtest_endpoint() {
    let state = build_test_state();
    let app = create_router(state);

    let req = Request::builder()
        .uri("/api/backtest/detailed/EURGBP")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let data: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(data["report"].is_object());
    assert!(data["trades"].is_array());
    assert!(data["equity_curve"].is_array());
}

#[tokio::test]
async fn test_api_server_audit_endpoints() {
    let state = build_test_state();
    let app = create_router(state);

    // 1. Audit Full
    let req = Request::builder()
        .uri("/api/audit/full")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let full_audit: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(full_audit["pairs"].as_array().unwrap().len() >= 6);
    assert!(full_audit["scorecard"]["total_score"].as_u64().unwrap() <= 28);
    assert_eq!(
        full_audit["scorecard"]["pillars"].as_array().unwrap().len(),
        7
    );
    assert_eq!(
        full_audit["walk_forward"]["wfer_pct"].as_f64().unwrap(),
        94.8
    );

    // 2. Audit Pair
    let req2 = Request::builder()
        .uri("/api/audit/pair/EURGBP")
        .body(Body::empty())
        .unwrap();
    let resp2 = app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let bytes2 = resp2.into_body().collect().await.unwrap().to_bytes();
    let pair_audit: Value = serde_json::from_slice(&bytes2).unwrap();
    assert_eq!(pair_audit["tier"], 1);
    assert_eq!(pair_audit["multiplier"].as_f64().unwrap(), 2.0);
    assert!(!pair_audit["trades"].as_array().unwrap().is_empty());
    assert!(!pair_audit["monthly_breakdown"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn test_market_symbols_and_ingestion_api() {
    let state = build_test_state();
    let app = create_router(state);

    // 1. Test GET /api/market/symbols
    let req = Request::builder()
        .uri("/api/market/symbols")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let symbols: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(symbols.len() >= 7);

    let gold = symbols
        .iter()
        .find(|s| s["symbol"]["base"] == "XAU" && s["symbol"]["quote"] == "USD");
    assert!(gold.is_some());
    let g = gold.unwrap();
    assert_eq!(g["tier"], 4);
    assert_eq!(g["multiplier"].as_f64().unwrap(), 0.5);
    assert!(g["is_available"].as_bool().unwrap());
    assert!(g["candle_count"].as_u64().unwrap() > 50000);
}
