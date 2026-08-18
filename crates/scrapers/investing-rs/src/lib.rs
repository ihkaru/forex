use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use tracing::info;

use domain::errors::DomainError;
use domain::ports::{EconomicCalendarPort, EconomicEvent, ImpactLevel};

#[allow(dead_code)]
pub struct InvestingScraper {
    client: Client,
}

impl Default for InvestingScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl InvestingScraper {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl EconomicCalendarPort for InvestingScraper {
    async fn fetch_events(&self, date: DateTime<Utc>) -> Result<Vec<EconomicEvent>, DomainError> {
        info!("Scraping Investing.com untuk tanggal {}", date);

        Ok(vec![EconomicEvent {
            title: "ECB Monetary Policy Statement".to_string(),
            country_currency: "EUR".to_string(),
            impact: ImpactLevel::High,
            event_time: date,
            forecast: None,
            previous: None,
            actual: None,
        }])
    }
}
