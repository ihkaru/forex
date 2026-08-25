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

    let market_adapter = Arc::new(RealHistoricalMarketAdapter::new());
    let broker_connector = Arc::new(broker_connector::BrokerConnector::new("MRG_MT4_Bridge"));
    // Aktifkan TCP Socket Bridge di port 5555 untuk menerima live stream dari MetaTrader 4 MRG
    broker_connector
        .clone()
        .start_tcp_listener("127.0.0.1", 5555);

    let mut router = application::services::MarketDataRouterService::new();
    router.register(market_adapter.clone());
    router.register_for(
        domain::models::MarketDataSource::MrgDemoMt4,
        broker_connector.clone(),
    );
    router.register_for(
        domain::models::MarketDataSource::MrgRealMt4,
        broker_connector.clone(),
    );
    router.register_for(
        domain::models::MarketDataSource::MrgMetaTrader4,
        broker_connector.clone(),
    );
    let router = Arc::new(router);

    let strategy = Arc::new(PolaNStrategy::default());
    let storage = Arc::new(storage_db::InMemoryStorage::new());

    let ingestion_service = Arc::new(application::services::MarketIngestionService::new(
        storage.clone(),
    ));

    let state = Arc::new(AppState {
        market_adapter,
        broker_connector,
        router,
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
