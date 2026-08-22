use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use tracing::info;

use domain::errors::DomainError;
use domain::ports::{EconomicCalendarPort, EconomicEvent, ImpactLevel};

#[allow(dead_code)]
pub struct ForexFactoryScraper {
    client: Client,
}

impl Default for ForexFactoryScraper {
    fn default() -> Self {
        // Fallback ke Client::new() jika builder gagal — non-fatal untuk scraper publik
        // (tidak seperti TF Publisher, ForexFactory tidak memerlukan user-agent spesifik untuk auth)
        #[allow(clippy::disallowed_methods)]
        // Justifikasi: ForexFactory scraper menggunakan public endpoint, client default acceptable
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl ForexFactoryScraper {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EconomicCalendarPort for ForexFactoryScraper {
    async fn fetch_events(&self, date: DateTime<Utc>) -> Result<Vec<EconomicEvent>, DomainError> {
        info!("Scraping kalender ForexFactory untuk tanggal: {}", date);

        // Contoh mock parser kalender ForexFactory
        Ok(vec![EconomicEvent {
            title: "US CPI MoM (Non-Farm Payrolls / Inflation)".to_string(),
            country_currency: "USD".to_string(),
            impact: ImpactLevel::High,
            event_time: date,
            forecast: Some("0.3%".to_string()),
            previous: Some("0.2%".to_string()),
            actual: None,
        }])
    }
}
