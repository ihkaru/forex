use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use tracing::info;

use domain::errors::DomainError;
use domain::models::Signal;
use domain::ports::{PublishReceipt, SignalPublisherPort};

pub struct TraderFamilyConfig {
    pub base_url: String,
    pub auth_token: String,
    pub channel_id: String,
    pub user_agent: String,
}

#[allow(dead_code)]
pub struct TraderFamilyPublisher {
    config: TraderFamilyConfig,
    client: Client,
}

impl TraderFamilyPublisher {
    pub fn new(config: TraderFamilyConfig) -> Result<Self, DomainError> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .build()
            // HTTP client yang gagal dibuild = tidak ada user-agent = TF API bisa reject request
            // WAJIB propagate, jangan fallback ke client tanpa konfigurasi
            .map_err(|e| {
                DomainError::AdapterError(format!(
                    "Gagal membuild HTTP client untuk TF Publisher: {}",
                    e
                ))
            })?;

        Ok(Self { config, client })
    }
}

#[async_trait]
impl SignalPublisherPort for TraderFamilyPublisher {
    fn platform_name(&self) -> &'static str {
        "trader-family"
    }

    async fn publish_signal(&self, signal: &Signal) -> Result<PublishReceipt, DomainError> {
        let formatted_text = signal.formatted_summary();
        info!(
            "Mengirim sinyal ke Trader Family Channel [{}]:\n{}",
            self.config.channel_id, formatted_text
        );

        // Contoh payload request REST API Trader Family (akan disesuaikan dengan hasil dump Jadx/Mitmproxy)
        let _payload = serde_json::json!({
            "channel_id": self.config.channel_id,
            "pair": signal.symbol.to_pair_string(),
            "action": format!("{:?}", signal.action),
            "entry_price": signal.entry_price.to_string(),
            "stop_loss": signal.stop_loss.to_string(),
            "take_profit_1": signal.take_profit_1.to_string(),
            "take_profit_2": signal.take_profit_2.map(|p| p.to_string()),
            "message": formatted_text,
        });

        // Mocking receipt hasil reverse engineering
        let receipt = PublishReceipt {
            signal_id: signal.id,
            channel_target: self.config.channel_id.clone(),
            external_post_id: format!("tf-post-{}", signal.id),
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
            "Mengupdate post Trader Family [{}] dengan status baru: {:?}",
            receipt.external_post_id, updated_signal.status
        );
        Ok(())
    }
}
