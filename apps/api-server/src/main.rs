use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use api_server::state::{AppState, RealHistoricalMarketAdapter};
use domain::models::PolaNStrategy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("🌐 Memulai Hexagon Quantitative REST API Server...");

    // Composition Root: Instantiate Adapters & Strategies
    let market_adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let strategy = Arc::new(PolaNStrategy::with_params(
        "TF-PolaN-Production",
        5,
        3,
        rust_decimal_macros::dec!(0.00020),
        rust_decimal_macros::dec!(1.3),
    ));
    let broker_connector = Arc::new(broker_connector::BrokerConnector::new("MRG_MT4_Bridge"));
    let storage = Arc::new(storage_db::InMemoryStorage::new());
    let ingestion_service = Arc::new(application::services::MarketIngestionService::new(
        storage.clone(),
    ));

    let state = Arc::new(AppState {
        market_adapter,
        broker_connector,
        strategy,
        storage,
        ingestion_service,
    });

    // Wire Routes & Ports via Router Factory
    let app = api_server::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    info!("🚀 REST API Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
