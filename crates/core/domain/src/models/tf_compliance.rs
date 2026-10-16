use chrono::{DateTime, Datelike, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::errors::DomainError;
use crate::models::signal::{Signal, SignalAction};
use crate::models::tf_spec::TfPairSpec;

/// Guard Validator Kepatuhan Sinyal terhadap Kebijakan Traders Family (100% Zero-Banned Guarantee)
pub struct TfComplianceGuard;

impl TfComplianceGuard {
    /// Validasi menyeluruh terhadap sinyal sebelum dipublikasikan ke channel Traders Family
    pub fn validate_signal(signal: &Signal) -> Result<(), DomainError> {
        // 1. Wajib Pending Order (Instant execution dilarang di TF App)
        match signal.action {
            SignalAction::BuyLimit
            | SignalAction::SellLimit
            | SignalAction::BuyStop
            | SignalAction::SellStop => {}
            SignalAction::Buy | SignalAction::Sell | SignalAction::ClosePosition => {
                return Err(DomainError::ValidationError(
                    "Sinyal TF wajib bertipe Pending Order (BuyLimit, SellLimit, BuyStop, SellStop)".to_string(),
                ));
            }
        }

        let spec = TfPairSpec::from_symbol(&signal.symbol);

        // 2. Hitung jarak SL dan TP dalam pips
        let sl_distance = (signal.entry_price - signal.stop_loss).abs();
        let tp_distance = (signal.take_profit_1 - signal.entry_price).abs();

        let sl_pips = spec.price_diff_to_pips(sl_distance);
        let tp_pips = spec.price_diff_to_pips(tp_distance);

        // 3. Validasi Batas Minimal & Maksimal Stop Loss
        if sl_pips < spec.min_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Stop Loss ({:.1} pips) di bawah batas minimal tier {} ({:.1} pips)",
                sl_pips, spec.symbol, spec.min_sl_tp_pips
            )));
        }
        if sl_pips > spec.max_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Stop Loss ({:.1} pips) melebihi batas maksimal tier {} ({:.1} pips)",
                sl_pips, spec.symbol, spec.max_sl_tp_pips
            )));
        }

        // 4. Validasi Batas Minimal & Maksimal Take Profit
        if tp_pips < spec.min_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Take Profit ({:.1} pips) di bawah batas minimal tier {} ({:.1} pips)",
                tp_pips, spec.symbol, spec.min_sl_tp_pips
            )));
        }
        if tp_pips > spec.max_sl_tp_pips {
            return Err(DomainError::ValidationError(format!(
                "Take Profit ({:.1} pips) melebihi batas maksimal tier {} ({:.1} pips)",
                tp_pips, spec.symbol, spec.max_sl_tp_pips
            )));
        }

        // 5. Validasi Maksimal Rasio Risk:Reward (Maksimal 1:3.0)
        let rr = tp_distance / sl_distance;
        if rr > dec!(3.00) {
            return Err(DomainError::ValidationError(format!(
                "Rasio Risk:Reward (1:{:.2}) melebihi batas maksimal TF (1:3.00)",
                rr
            )));
        }
        if rr < dec!(1.00) {
            return Err(DomainError::ValidationError(format!(
                "Rasio Risk:Reward (1:{:.2}) tidak sehat (wajib minimal 1:1.00)",
                rr
            )));
        }

        // 6. Validasi Stop Loss maksimal 1.5x dari Take Profit
        if sl_pips > (tp_pips * dec!(1.5)) {
            return Err(DomainError::ValidationError(format!(
                "Stop Loss ({:.1} pips) melebihi 1.5x Take Profit ({:.1} pips)",
                sl_pips, tp_pips
            )));
        }

        // 7. Validasi Masa Kadaluwarsa (Expired Time: 1 jam s.d. 48 jam / 96 jam Jumat)
        if let Some(expires_at) = signal.expires_at {
            Self::validate_expiration(signal.created_at, expires_at)?;
        }

        Ok(())
    }

    /// Validasi durasi kedaluwarsa sesuai hari pembuatan
    pub fn validate_expiration(
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let duration_hours = (expires_at - created_at).num_hours();
        let is_friday = created_at.weekday() == chrono::Weekday::Fri;
        let max_hours = if is_friday { 96 } else { 48 };

        if duration_hours < 1 {
            return Err(DomainError::ValidationError(
                "Durasi expired sinyal minimal adalah 1 jam".to_string(),
            ));
        }
        if duration_hours > max_hours {
            return Err(DomainError::ValidationError(format!(
                "Durasi expired sinyal ({} jam) melebihi batas maksimal ({} jam)",
                duration_hours, max_hours
            )));
        }

        Ok(())
    }

    /// Validasi kuota maksimal 2 sinyal aktif per pair dan kepatuhan anti-martingale (jarak pending searah)
    /// serta penjagaan idempotensi sinyal (mencegah penumpukan duplicate limit/stop order)
    pub fn validate_concurrent_signals(
        new_signal: &Signal,
        active_signals: &[Signal],
    ) -> Result<(), DomainError> {
        let active_for_symbol: Vec<&Signal> = active_signals
            .iter()
            .filter(|s| s.symbol == new_signal.symbol)
            .collect();

        // 1. Invariant TF Rule #4: Maksimal 2 sinyal aktif per pair
        if active_for_symbol.len() >= 2 {
            return Err(DomainError::ValidationError(format!(
                "Kuota sinyal aktif untuk pair {} sudah penuh (maksimal 2 sinyal bersamaan)",
                new_signal.symbol
            )));
        }

        let spec = TfPairSpec::from_symbol(&new_signal.symbol);

        for existing in active_for_symbol {
            // 2. Idempotensi Mutlak (FIX Tag 11 / ClOrdID): Dilarang order identik di level harga yang sama
            if existing.action == new_signal.action
                && existing.entry_price == new_signal.entry_price
            {
                return Err(DomainError::ValidationError(format!(
                    "Sinyal duplikat terdeteksi (Idempotency check failed): Sinyal {:?} di level {} untuk {} sudah aktif",
                    new_signal.action, new_signal.entry_price, new_signal.symbol
                )));
            }

            // 3. Jarak Pending Searah (Anti-Martingale TF Rule #5):
            // Tier 1 >= 50 pips, Tier 2 >= 75 pips, Tier 3/4 >= 100 pips
            if existing.action == new_signal.action {
                let diff = (new_signal.entry_price - existing.entry_price).abs();
                let diff_pips = spec.price_diff_to_pips(diff);

                if diff_pips < spec.min_same_direction_gap_pips {
                    return Err(DomainError::ValidationError(format!(
                        "Pelanggaran Anti-Martingale TF: Jarak pending order searah ({:.1} pips) kurang dari batas minimal tier {} ({:.1} pips)",
                        diff_pips, spec.symbol, spec.min_same_direction_gap_pips
                    )));
                }
            }
        }

        Ok(())
    }
}
