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
    let strategy = Arc::new(PolaNStrategy::default());
    Arc::new(AppState {
        market_adapter,
        strategy,
    })
}

#[tokio::test]
async fn test_market_candles_api_returns_real_historical_data() {
    let state = build_test_state();
    let app = create_router(state);

    let pairs = ["EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD"];

    for pair in pairs {
        let req = Request::builder()
            .uri(format!("/api/market/candles/{}", pair))
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
    assert_eq!(scorecard["total_score"], 28);
    assert_eq!(scorecard["channel_level"], "LEGEND");
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
    assert!(mc["equity_paths"].as_array().unwrap().len() > 0);
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
