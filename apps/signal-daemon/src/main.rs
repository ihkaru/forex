use std::sync::Arc;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use application::services::SignalEngineService;
use broker_connector::BrokerConnector;
use domain::models::{AppConfig, PolaNStrategy, RiskProfile, Symbol, Timeframe};
use domain::ports::{SignalPublisherPort, StrategyPort};
use notifier_telegram::{TelegramConfig, TelegramNotifier};
use publisher_traderfamily::{TraderFamilyConfig, TraderFamilyPublisher};
use storage_db::InMemoryStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Inisialisasi Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Memulai Forex Autonomous Signal Daemon (Traders Family Engine)...");

    // 2. Load Konfigurasi Terpusat dari config.toml / Environment Variables
    let config = AppConfig::load_from_file_or_default("config.toml");
    info!(
        "Konfigurasi aktif dimuat (Env: {}, TF Base URL: {}, Active Pairs: {})",
        config.environment,
        config.traders_family.api_base_url,
        config.active_symbols.len()
    );

    // 3. Inisialisasi Adapters (Composition Root)
    let broker_name = format!(
        "{}:{}",
        config.broker.mt5_socket_host, config.broker.mt5_socket_port
    );
    let market_data = Arc::new(BrokerConnector::new(&broker_name));
    market_data.clone().start_tcp_listener(
        &config.broker.mt5_socket_host,
        config.broker.mt5_socket_port,
    );

    let tf_config = TraderFamilyConfig {
        base_url: config.traders_family.api_base_url.clone(),
        auth_token: config.traders_family.auth_token.clone(),
        channel_id: config.traders_family.channel_id.clone(),
        user_agent: config.traders_family.user_agent.clone(),
    };
    let tf_publisher: Arc<dyn SignalPublisherPort> =
        Arc::new(TraderFamilyPublisher::new(tf_config)?);

    let mut publishers: Vec<Arc<dyn SignalPublisherPort>> = vec![tf_publisher];

    // Opsional: Telegram fallback jika diaktifkan di config
    if config.telegram.enabled && !config.telegram.bot_token.is_empty() {
        let tg_config = TelegramConfig {
            bot_token: config.telegram.bot_token.clone(),
            chat_id: config.telegram.chat_id.clone(),
        };
        let tg_notifier: Arc<dyn SignalPublisherPort> = Arc::new(TelegramNotifier::new(tg_config));
        publishers.push(tg_notifier);
        info!("📢 Notifier Telegram multi-channel diaktifkan.");
    }

    let storage = Arc::new(InMemoryStorage::new());

    // 4. Inisialisasi Strategi Berbasis Config
    let pola_n_strategy: Arc<dyn StrategyPort> =
        Arc::new(PolaNStrategy::from_config(&config.strategy_pola_n));
    let strategies: Vec<Arc<dyn StrategyPort>> = vec![pola_n_strategy];

    let risk_profile = RiskProfile::from_config(&config.risk_management);

    // 5. Inisialisasi Application Engine dengan Invariant Kepatuhan TF
    let engine =
        SignalEngineService::new(market_data, publishers, storage, strategies, risk_profile);

    let watched_symbols: Vec<Symbol> = config
        .active_symbols
        .iter()
        .filter_map(|s| Symbol::from_symbol_str(s))
        .collect();

    let timeframe = match config.default_timeframe.as_str() {
        "M1" => Timeframe::M1,
        "M5" => Timeframe::M5,
        "M15" => Timeframe::M15,
        "M30" => Timeframe::M30,
        "H1" => Timeframe::H1,
        "H4" => Timeframe::H4,
        "D1" => Timeframe::D1,
        _ => Timeframe::H1,
    };

    info!("Memantau pasang mata uang aktif: {:?}", watched_symbols);

    // 6. Main Continuous Autonomous Loop
    let poll_interval = Duration::from_secs(10);
    info!(
        "🔄 Memulai continuous monitoring loop (Interval: {:?})",
        poll_interval
    );

    loop {
        for symbol in &watched_symbols {
            match engine.process_symbol(symbol, timeframe).await {
                Ok(Some(sig)) => {
                    info!("🎯 Sinyal aktif lolos kepatuhan TF & diposting: {}", sig.id);
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("⚠️ Error evaluasi simbol {}: {}", symbol, e);
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        tokio::time::sleep(poll_interval).await;
    }
}
