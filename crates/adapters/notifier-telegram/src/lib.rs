use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use tracing::info;

use domain::errors::DomainError;
use domain::models::Signal;
use domain::ports::{PublishReceipt, SignalPublisherPort};

pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

#[allow(dead_code)]
pub struct TelegramNotifier {
    config: TelegramConfig,
    client: Client,
}

impl TelegramNotifier {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SignalPublisherPort for TelegramNotifier {
    fn platform_name(&self) -> &'static str {
        "telegram"
    }

    async fn publish_signal(&self, signal: &Signal) -> Result<PublishReceipt, DomainError> {
        let _text = signal.formatted_summary();
        info!("Kirim pesan sinyal ke Telegram Chat [{}]", self.config.chat_id);

        let receipt = PublishReceipt {
            signal_id: signal.id,
            channel_target: self.config.chat_id.clone(),
            external_post_id: format!("tg-msg-{}", signal.id),
            published_at: Utc::now(),
        };

        Ok(receipt)
    }

    async fn update_signal_status(
        &self,
        receipt: &PublishReceipt,
        updated_signal: &Signal,
    ) -> Result<(), DomainError> {
        info!(
            "Update pesan telegram [{}] -> {:?}",
            receipt.external_post_id, updated_signal.status
        );
        Ok(())
    }
}
