use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use tokio::sync::RwLock;
use tracing::info;

use domain::errors::DomainError;
use domain::models::Signal;
use domain::ports::{PublishReceipt, SignalPublisherPort};

#[derive(Debug, Clone)]
pub struct TraderFamilyConfig {
    pub base_url: String,
    pub auth_token: String,
    pub channel_id: String,
    pub user_agent: String,
    pub email: String,
    pub password: String,
}

pub struct TraderFamilyPublisher {
    config: TraderFamilyConfig,
    client: Client,
    active_token: RwLock<Option<String>>,
}

impl TraderFamilyPublisher {
    pub fn new(config: TraderFamilyConfig) -> Result<Self, DomainError> {
        if config.auth_token.to_lowercase().contains("mock")
            || config.email.to_lowercase().contains("mock")
            || config.channel_id.to_lowercase().contains("mock")
        {
            return Err(DomainError::AdapterError(
                "FAIL-FAST: Konfigurasi Traders Family mengandung string 'mock'! Dilarang menggunakan mock di jalur produksi.".to_string(),
            ));
        }

        let client = Client::builder()
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| {
                DomainError::AdapterError(format!(
                    "Gagal membuild HTTP client untuk TF Publisher: {}",
                    e
                ))
            })?;

        let initial_token = if !config.auth_token.is_empty() {
            Some(config.auth_token.clone())
        } else {
            None
        };

        Ok(Self {
            config,
            client,
            active_token: RwLock::new(initial_token),
        })
    }

    /// Mengambil token aktif yang valid, atau melakukan auto-login jika belum ada
    pub async fn get_valid_token(&self) -> Result<String, DomainError> {
        // 1. Cek cached in-memory token
        {
            let guard = self.active_token.read().await;
            if let Some(ref tok) = *guard {
                if !tok.is_empty() {
                    return Ok(tok.clone());
                }
            }
        }

        // 2. Cek token dari config jika ada
        if !self.config.auth_token.is_empty() {
            let mut guard = self.active_token.write().await;
            *guard = Some(self.config.auth_token.clone());
            return Ok(self.config.auth_token.clone());
        }

        // 3. Auto-login via email & password ke gateway Traders Family
        if !self.config.email.is_empty() && !self.config.password.is_empty() {
            let token = self.login_with_credentials().await?;
            let mut guard = self.active_token.write().await;
            *guard = Some(token.clone());
            return Ok(token);
        }

        Err(DomainError::AdapterError(
            "Tidak ada auth_token atau email/password yang dikonfigurasi untuk Traders Family"
                .to_string(),
        ))
    }

    /// Melakukan HTTP POST /traders/api/v1/login/ dan mengembalikan Bearer JWT Token
    pub async fn login_with_credentials(&self) -> Result<String, DomainError> {
        let login_url = format!(
            "{}/traders/api/v1/login/",
            self.config.base_url.trim_end_matches('/')
        );
        info!(
            "🔐 Melakukan auto-login ke Traders Family Gateway: {}",
            login_url
        );

        let response = self
            .client
            .post(&login_url)
            .json(&serde_json::json!({
                "email": self.config.email,
                "password": self.config.password,
            }))
            .send()
            .await
            .map_err(|e| {
                DomainError::AdapterError(format!("Network error saat login TF: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.map_err(|e| {
                DomainError::AdapterError(format!("Gagal membaca body error login TF: {}", e))
            })?;
            return Err(DomainError::AdapterError(format!(
                "Login TF gagal HTTP {}: {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await.map_err(|e| {
            DomainError::AdapterError(format!("Parse login JSON response gagal: {}", e))
        })?;

        // Format respon: { "result": "eyJhbGci...", "user": { ... } }
        let token = body
            .get("result")
            .and_then(|v| v.as_str())
            .or_else(|| {
                body.get("message")
                    .and_then(|m| m.get("result"))
                    .and_then(|v| v.as_str())
            })
            .ok_or_else(|| {
                DomainError::AdapterError(
                    "Field 'result' (JWT token) tidak ditemukan dalam respons login TF".to_string(),
                )
            })?;

        info!("✅ Auto-login Traders Family berhasil! JWT Session Token diperoleh.");
        Ok(token.to_string())
    }
}

#[async_trait]
impl SignalPublisherPort for TraderFamilyPublisher {
    fn platform_name(&self) -> &'static str {
        "trader-family"
    }

    async fn publish_signal(&self, signal: &Signal) -> Result<PublishReceipt, DomainError> {
        let token = self.get_valid_token().await?;
        let formatted_text = signal.formatted_summary();

        info!(
            "🚀 Mengirim sinyal ke Trader Family Channel [{}]:\n{}",
            self.config.channel_id, formatted_text
        );

        // Mapping Action Pola N ke Op Code Traders Family:
        // 0: Buy Limit, 1: Sell Limit, 2: Buy Stop, 3: Sell Stop
        let op_code = match signal.action {
            domain::models::SignalAction::BuyLimit | domain::models::SignalAction::Buy => 0,
            domain::models::SignalAction::SellLimit | domain::models::SignalAction::Sell => 1,
            domain::models::SignalAction::BuyStop => 2,
            domain::models::SignalAction::SellStop => 3,
            domain::models::SignalAction::ClosePosition => 0,
        };

        let create_signal_url = format!(
            "{}/ois/api/v1/signal/create/",
            self.config.base_url.trim_end_matches('/')
        );

        let payload = serde_json::json!({
            "channel_id": self.config.channel_id,
            "symbol": signal.symbol.to_pair_string(),
            "op": op_code,
            "open_price": signal.entry_price.to_string(),
            "sl": signal.stop_loss.to_string(),
            "tp": signal.take_profit_1.to_string(),
            "lot": 0.01,
            "expired": 24,
            "description": formatted_text,
        });

        let response = self
            .client
            .post(&create_signal_url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                DomainError::AdapterError(format!("Network error create signal TF: {}", e))
            })?;

        let status = response.status();
        let resp_text = response.text().await.map_err(|e| {
            DomainError::AdapterError(format!("Gagal membaca body create signal TF: {}", e))
        })?;

        if !status.is_success() {
            return Err(DomainError::AdapterError(format!(
                "Gagal publish sinyal ke TF HTTP {}: {}",
                status, resp_text
            )));
        }

        info!(
            "🎯 Respon server Traders Family saat publikasi sinyal: {}",
            resp_text
        );

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text).map_err(|e| {
            DomainError::AdapterError(format!(
                "FAIL-FAST: Respon server Traders Family bukan format JSON valid: {} | Body: {}",
                e, resp_text
            ))
        })?;

        // Validasi response code dari TF microservice
        if let Some(code) = resp_json.get("code").and_then(|c| c.as_i64()) {
            if code != 200 {
                let msg = resp_json
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Penolakan oleh server TF");
                return Err(DomainError::AdapterError(format!(
                    "FAIL-FAST: Server Traders Family menolak publikasi sinyal (code {}): {}",
                    code, msg
                )));
            }
        }

        // Ekstraksi External Signal ID riil dari respons server TF
        let external_id = resp_json
            .get("result")
            .and_then(|r| {
                r.get("id")
                    .and_then(|id| {
                        id.as_i64()
                            .map(|n| n.to_string())
                            .or_else(|| id.as_str().map(|s| s.to_string()))
                    })
                    .or_else(|| r.as_i64().map(|n| n.to_string()))
                    .or_else(|| r.as_str().map(|s| s.to_string()))
            })
            .or_else(|| {
                resp_json.get("data").and_then(|d| {
                    d.get("id")
                        .and_then(|id| {
                            id.as_i64()
                                .map(|n| n.to_string())
                                .or_else(|| id.as_str().map(|s| s.to_string()))
                        })
                        .or_else(|| d.as_i64().map(|n| n.to_string()))
                        .or_else(|| d.as_str().map(|s| s.to_string()))
                })
            })
            .or_else(|| {
                resp_json.get("id").and_then(|id| {
                    id.as_i64()
                        .map(|n| n.to_string())
                        .or_else(|| id.as_str().map(|s| s.to_string()))
                })
            })
            .ok_or_else(|| {
                DomainError::AdapterError(format!(
                    "FAIL-FAST: Respon server TF tidak memuat ID sinyal riil: {}",
                    resp_text
                ))
            })?;

        info!(
            "✅ Sinyal sukses diposting ke Traders Family! Real Signal ID: {}",
            external_id
        );

        let receipt = PublishReceipt {
            signal_id: signal.id,
            channel_target: self.config.channel_id.clone(),
            external_post_id: external_id,
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
