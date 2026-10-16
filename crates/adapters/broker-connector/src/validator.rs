use chrono::{DateTime, TimeZone, Utc};
use domain::errors::DomainError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::warn;

/// Validasi Integritas Data & Sanitizer
pub struct DataIntegrityValidator;

impl DataIntegrityValidator {
    /// Validasi integritas harga Tick (Anti-Spike & Valid Spread)
    pub fn validate_tick(bid: Decimal, ask: Decimal) -> Result<(), DomainError> {
        if bid <= Decimal::ZERO || ask <= Decimal::ZERO {
            return Err(DomainError::InvalidPrice("Harga harus > 0".to_string()));
        }
        if ask < bid {
            return Err(DomainError::InvalidPrice(
                "Integritas Rusak: Ask lebih kecil dari Bid (Negative Spread)".to_string(),
            ));
        }
        let spread = ask - bid;
        // Kalibrasi peringatan anomali spread: Forex > 100 pips (0.0100) atau Gold > 500 pips ($5.00)
        let is_anomaly = if bid > dec!(100.0) {
            spread > dec!(5.00)
        } else {
            spread > dec!(0.0100)
        };
        if is_anomaly {
            warn!(
                "⚠️ Peringatan Anomali Pasar: Spread melebar drastis ({})",
                spread
            );
        }
        Ok(())
    }

    /// Konversi timestamp broker ke standar waktu internasional UTC
    pub fn normalize_to_utc(timestamp_seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(timestamp_seconds, 0)
            .single()
            .unwrap_or_else(Utc::now)
    }
}
