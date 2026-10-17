use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use api_server::state::AppState;
use domain::models::PolaNStrategy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("🌐 Memulai Hexagon Quantitative REST API Server...");

    // Inisialisasi Persistent Storage Engine Agnostik
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/forex.db".to_string());
    let (storage, tick_storage, broker_spec_storage): (
        Arc<dyn domain::ports::StoragePort>,
        Arc<dyn domain::ports::TickStoragePort>,
        Arc<dyn domain::ports::BrokerSpecPort>,
    ) = if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        info!("📦 Menggunakan PostgreSQL/TimescaleDB Storage");
        let pg = Arc::new(storage_db::SqlxStorage::connect(&db_url).await?);
        (pg.clone(), pg.clone(), pg)
    } else {
        let path = if let Some(stripped) = db_url.strip_prefix("sqlite://") {
            stripped
        } else {
            &db_url
        };
        info!("📦 Menggunakan Embedded SQLite Storage di: {}", path);
        let sq = Arc::new(storage_db::SqliteStorage::connect(path).await?);
        (sq.clone(), sq.clone(), sq)
    };

    let market_adapter: Arc<dyn domain::ports::MarketDataPort> = Arc::new(
        storage_db::StorageMarketDataAdapter::with_tick_and_spec_storage(
            domain::models::MarketDataSource::DukascopyEcn,
            storage.clone(),
            Some(tick_storage.clone()),
            Some(broker_spec_storage.clone()),
        ),
    );
    let broker_connector = Arc::new(
        broker_connector::BrokerConnector::new("MRG_MT4_Bridge")
            .with_storage(storage.clone(), tick_storage.clone(), broker_spec_storage.clone())
    );
    // Aktifkan TCP Socket Bridge di port 5555 untuk menerima live stream dari MetaTrader 4
    let mt4_host = std::env::var("MT4_SOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let mt4_port = std::env::var("MT4_SOCKET_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5555);
    broker_connector
        .clone()
        .start_tcp_listener(&mt4_host, mt4_port);

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

    let ingestion_service = Arc::new(application::services::MarketIngestionService::new(
        storage.clone(),
    ));

    // Seeding idempoten data pasar historis dari disk ke storage saat pertama kali boot
    let default_pairs = [
        "EURGBP", "USDCHF", "GBPUSD", "EURUSD", "NZDUSD", "AUDUSD", "XAUUSD", "USDCAD",
        "USDJPY", "EURJPY", "GBPJPY",
    ];
    for p in &default_pairs {
        if let Some(sym) = domain::models::Symbol::from_symbol_str(p) {
            let _ = ingestion_service.seed_storage_from_disk(&sym).await;
        }
    }

    let state = Arc::new(AppState {
        market_adapter,
        broker_connector,
        router,
        strategy,
        storage,
        tick_storage,
        broker_spec_storage,
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
