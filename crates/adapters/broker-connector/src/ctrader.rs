use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{Candle, Symbol, Tick, Timeframe};
use domain::ports::MarketDataPort;

// ==============================================================================
// cTRADER OPEN API CONNECTOR (Headless Protobuf Stream / Zero GUI Dependency)
// ==============================================================================

pub struct CtraderOpenApiConnector {
    pub client_id: String,
    pub client_secret: String,
    pub environment: String, // "demo.ctraderapi.com" or "live.ctraderapi.com"
}

impl CtraderOpenApiConnector {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        is_live: bool,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            environment: if is_live {
                "live.ctraderapi.com".to_string()
            } else {
                "demo.ctraderapi.com".to_string()
            },
        }
    }
}

#[async_trait]
impl MarketDataPort for CtraderOpenApiConnector {
    fn source(&self) -> domain::models::MarketDataSource {
        domain::models::MarketDataSource::CtraderOpenApi
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        // FAIL-FAST: Dilarang fallback ke snapshot mock!
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: cTrader Open API stream belum terkoneksi untuk simbol {}. Menolak fake tick.",
            symbol
        )))
    }

    async fn get_recent_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        _limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        // FAIL-FAST: Dilarang membuat lilin sintetis palsu!
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: cTrader Open API belum menerima streaming candle untuk simbol {} timeframe {:?}. Menolak synthetic candles.",
            symbol, timeframe
        )))
    }

    async fn get_historical_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        self.get_recent_candles(symbol, timeframe, 500).await
    }
}
