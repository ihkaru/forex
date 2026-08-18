use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::Signal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub signal_id: Uuid,
    pub channel_target: String,
    pub external_post_id: String,
    pub published_at: DateTime<Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SignalPublisherPort: Send + Sync {
    /// Nama platform publisher (contoh: "trader-family", "telegram")
    fn platform_name(&self) -> &'static str;

    /// Publikasikan sinyal baru ke subscriber channel
    async fn publish_signal(&self, signal: &Signal) -> Result<PublishReceipt, DomainError>;

    /// Update status sinyal yang sedang berjalan (Hit TP/SL/Close)
    async fn update_signal_status(
        &self,
        receipt: &PublishReceipt,
        updated_signal: &Signal,
    ) -> Result<(), DomainError>;
}
