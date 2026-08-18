use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("Invalid symbol format: {0}")]
    InvalidSymbol(String),

    #[error("Invalid price value: {0}")]
    InvalidPrice(String),

    #[error("Invalid stop loss or take profit ratio: {0}")]
    InvalidRiskReward(String),

    #[error("Strategy calculation error: {0}")]
    StrategyError(String),

    #[error("Port adapter failure: {0}")]
    AdapterError(String),

    #[error("Scraper failure: {0}")]
    ScraperError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),
}
