use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Konfigurasi Utama Ekosistem Kuantitatif (AppConfig)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default)]
    pub traders_family: TradersFamilySettings,

    #[serde(default)]
    pub telegram: TelegramSettings,

    #[serde(default)]
    pub broker: BrokerSettings,

    #[serde(default = "default_active_symbols")]
    pub active_symbols: Vec<String>,

    #[serde(default = "default_timeframe_str")]
    pub default_timeframe: String,

    #[serde(default)]
    pub strategy_pola_n: PolaNConfig,

    #[serde(default)]
    pub risk_management: RiskConfig,

    #[serde(default)]
    pub backtest: BacktestConfig,
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_timeframe_str() -> String {
    "H1".to_string()
}

fn default_active_symbols() -> Vec<String> {
    vec![
        "NZDUSD".to_string(),
        "AUDUSD".to_string(),
        "EURGBP".to_string(),
        "USDCHF".to_string(),
        "EURUSD".to_string(),
        "GBPUSD".to_string(),
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: default_environment(),
            traders_family: TradersFamilySettings::default(),
            telegram: TelegramSettings::default(),
            broker: BrokerSettings::default(),
            active_symbols: default_active_symbols(),
            default_timeframe: default_timeframe_str(),
            strategy_pola_n: PolaNConfig::default(),
            risk_management: RiskConfig::default(),
            backtest: BacktestConfig::default(),
        }
    }
}

/// Pengaturan API Channel Traders Family
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradersFamilySettings {
    pub api_base_url: String,
    pub channel_id: String,
    pub auth_token: String,
    pub user_agent: String,
}

impl Default for TradersFamilySettings {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.tradersfamily.id".to_string(),
            channel_id: "tf_priority_quant_channel".to_string(),
            auth_token: String::new(),
            user_agent: "TradersFamily-Android/3.0".to_string(),
        }
    }
}

/// Pengaturan Notifikasi Telegram
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TelegramSettings {
    pub enabled: bool,
    pub bot_token: String,
    pub chat_id: String,
}

/// Pengaturan Koneksi Broker (MetaTrader 5 Bridge EA Socket)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrokerSettings {
    pub mt5_socket_host: String,
    pub mt5_socket_port: u16,
    pub reconnect_interval_ms: u64,
}

impl Default for BrokerSettings {
    fn default() -> Self {
        Self {
            mt5_socket_host: "127.0.0.1".to_string(),
            mt5_socket_port: 8080,
            reconnect_interval_ms: 3000,
        }
    }
}

/// Parameter Strategi Pola N (Traders Family Signature)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolaNConfig {
    pub swing_left_bars: usize,
    pub swing_right_bars: usize,
    pub pip_buffer_pips: Decimal,
    pub min_risk_reward: Decimal,
    pub max_risk_reward: Decimal,
    pub default_expiration_hours: i64,
}

impl Default for PolaNConfig {
    fn default() -> Self {
        Self {
            swing_left_bars: 2,
            swing_right_bars: 2,
            pip_buffer_pips: dec!(2.0),
            min_risk_reward: dec!(2.0),
            max_risk_reward: dec!(3.0),
            default_expiration_hours: 24,
        }
    }
}

/// Parameter Manajemen Risiko Global
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_risk_per_trade_percent: Decimal,
    pub min_risk_reward_ratio: Decimal,
    pub max_open_drawdown_percent: Decimal,
    pub max_spread_pips: Decimal,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_risk_per_trade_percent: dec!(1.0),
            min_risk_reward_ratio: dec!(2.0),
            max_open_drawdown_percent: dec!(5.0),
            max_spread_pips: dec!(2.5),
        }
    }
}

/// Parameter Simulasi Backtesting Lab
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub simulation_spread_pips: Decimal,
    pub lookback_window_bars: usize,
    pub default_lot_size: Decimal,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            simulation_spread_pips: dec!(1.2),
            lookback_window_bars: 50,
            default_lot_size: dec!(1.0),
        }
    }
}

impl AppConfig {
    /// Membaca konfigurasi dari string format TOML
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Membaca konfigurasi dari file `config.toml` di path tertentu dengan fallback ke Default
    pub fn load_from_file_or_default(path: &str) -> Self {
        if let Ok(content) = std::fs::read_to_string(path) {
            match Self::from_toml_str(&content) {
                Ok(mut config) => {
                    // Override token dari environment variable jika tersedia
                    if let Ok(token) = std::env::var("TF_AUTH_TOKEN") {
                        if !token.is_empty() {
                            config.traders_family.auth_token = token;
                        }
                    }
                    if let Ok(tg_token) = std::env::var("TELEGRAM_BOT_TOKEN") {
                        if !tg_token.is_empty() {
                            config.telegram.bot_token = tg_token;
                        }
                    }
                    return config;
                }
                Err(e) => {
                    eprintln!(
                        "⚠️ Gagal mem-parse file config di '{}': {}. Menggunakan Default config.",
                        path, e
                    );
                }
            }
        }
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_toml_parsing() {
        let toml_data = r#"
            environment = "production"
            active_symbols = ["NZDUSD", "AUDUSD"]

            [traders_family]
            api_base_url = "https://api.tradersfamily.id"
            channel_id = "vip_channel"
            auth_token = "secret123"
            user_agent = "TradersFamily-Android/3.0"

            [strategy_pola_n]
            swing_left_bars = 3
            swing_right_bars = 3
            pip_buffer_pips = 2.5
            min_risk_reward = 2.0
            max_risk_reward = 3.0
            default_expiration_hours = 48
        "#;

        let config: AppConfig = AppConfig::from_toml_str(toml_data).unwrap();
        assert_eq!(config.environment, "production");
        assert_eq!(config.active_symbols, vec!["NZDUSD", "AUDUSD"]);
        assert_eq!(config.traders_family.channel_id, "vip_channel");
        assert_eq!(config.strategy_pola_n.swing_left_bars, 3);
        assert_eq!(config.strategy_pola_n.pip_buffer_pips, dec!(2.5));
    }
}
