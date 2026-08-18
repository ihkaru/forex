use chrono::Utc;
use domain::models::Symbol;
use domain::ports::{EconomicCalendarPort, SentimentPort};
use forexfactory_rs::ForexFactoryScraper;
use myfxbook_rs::MyfxbookScraper;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🕷️ Menjalankan Scraper Background Worker...");

    let ff_scraper = ForexFactoryScraper::new();
    let events = ff_scraper.fetch_events(Utc::now()).await?;
    info!("ForexFactory Events ditemukan: {}", events.len());

    let sentiment_scraper = MyfxbookScraper::new();
    let symbol = Symbol::new("EUR", "USD");
    let sentiment = sentiment_scraper.fetch_sentiment(&symbol).await?;
    info!(
        "Sentimen EUR/USD: Long {}% | Short {}%",
        sentiment.long_percentage, sentiment.short_percentage
    );

    Ok(())
}
