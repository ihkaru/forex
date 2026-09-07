use async_trait::async_trait;
use reqwest::Client;
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

        // FAIL-FAST: Dilarang menggunakan sentimen retail mock!
        Err(DomainError::DataUnavailable(format!(
            "FAIL-FAST: Live retail sentiment scraper Myfxbook belum menerima data riil untuk {}. Menolak fake sentiment.",
            symbol
        )))
    }
}
