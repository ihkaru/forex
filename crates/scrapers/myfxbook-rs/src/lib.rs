use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal_macros::dec;
use tracing::info;

use domain::errors::DomainError;
use domain::models::Symbol;
use domain::ports::{SentimentData, SentimentPort};

#[allow(dead_code)]
pub struct MyfxbookScraper {
    client: Client,
}

impl Default for MyfxbookScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl MyfxbookScraper {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SentimentPort for MyfxbookScraper {
    async fn fetch_sentiment(&self, symbol: &Symbol) -> Result<SentimentData, DomainError> {
        info!("Scraping sentimen retail Myfxbook untuk {}", symbol);

        // Mocking rasio sentimen retail (misal EUR/USD 65% retail Buy -> Market maker sell)
        Ok(SentimentData {
            symbol: symbol.clone(),
            long_percentage: dec!(65.0),
            short_percentage: dec!(35.0),
            total_positions: 14200,
            fetched_at: Utc::now(),
        })
    }
}
