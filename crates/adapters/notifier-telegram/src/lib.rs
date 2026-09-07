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
    pub fn new(config: TelegramConfig) -> Result<Self, DomainError> {
        if config.bot_token.to_lowercase().contains("mock")
            || config.chat_id.to_lowercase().contains("mock")
        {
            return Err(DomainError::AdapterError(
                "FAIL-FAST: Konfigurasi Telegram mengandung string 'mock'! Dilarang menggunakan mock.".to_string(),
            ));
        }

        Ok(Self {
            config,
            client: Client::new(),
        })
    }
}

#[async_trait]
impl SignalPublisherPort for TelegramNotifier {
    fn platform_name(&self) -> &'static str {
        "telegram"
    }

    async fn publish_signal(&self, signal: &Signal) -> Result<PublishReceipt, DomainError> {
        if self.config.bot_token.is_empty() || self.config.chat_id.is_empty() {
            return Err(DomainError::AdapterError(
                "FAIL-FAST: Telegram bot_token atau chat_id kosong!".to_string(),
            ));
        }

        let text = signal.formatted_summary();
        info!(
            "Kirim pesan sinyal live ke Telegram Chat [{}]",
            self.config.chat_id
        );

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let payload = serde_json::json!({
            "chat_id": self.config.chat_id,
            "text": text,
            "parse_mode": "Markdown",
        });

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                DomainError::AdapterError(format!("Network error kirim Telegram message: {}", e))
            })?;

        let status = resp.status();
        let resp_text = resp.text().await.map_err(|e| {
            DomainError::AdapterError(format!("Gagal membaca body respons Telegram: {}", e))
        })?;

        if !status.is_success() {
            return Err(DomainError::AdapterError(format!(
                "FAIL-FAST: Telegram API error HTTP {}: {}",
                status, resp_text
            )));
        }

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text).map_err(|e| {
            DomainError::AdapterError(format!(
                "FAIL-FAST: Parse JSON respon Telegram gagal: {} | Body: {}",
                e, resp_text
            ))
        })?;

        let message_id = resp_json
            .get("result")
            .and_then(|r| r.get("message_id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                DomainError::AdapterError(format!(
                    "FAIL-FAST: Respon Telegram tidak memuat result.message_id: {}",
                    resp_text
                ))
            })?;

        let receipt = PublishReceipt {
            signal_id: signal.id,
            channel_target: self.config.chat_id.clone(),
            external_post_id: message_id.to_string(),
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
