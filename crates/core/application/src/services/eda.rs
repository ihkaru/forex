use chrono::{DateTime, Datelike, Utc};
use rust_decimal::Decimal;

use domain::models::{Candle, Symbol, TfPairSpec};

/// Laporan Hasil Exploratory Data Analysis (EDA) pada Deret Waktu Candlestick
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EdaReport {
    pub symbol: Symbol,
    pub total_candles: usize,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_duration_days: f64,
    
    // Integritas Matematika OHLCV
    pub invalid_candle_count: usize,
    pub mathematical_integrity_pct: f64,
    
    // Statistik Harga & Volatilitas (dalam pips)
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub avg_bar_range_pips: Decimal,
    pub max_single_bar_pips: Decimal,
    
    // Deteksi Gap & Outlier
    pub weekday_gaps_count: usize,
    pub zero_volume_bars_count: usize,
    
    // Status Kelayakan Data
    pub health_score: f64,
    pub health_status: String,
}

pub struct EdaService;

impl EdaService {
    /// Menjalankan Exploratory Data Analysis menyeluruh pada dataset candle
    pub fn analyze(symbol: &Symbol, candles: &[Candle]) -> EdaReport {
        if candles.is_empty() {
            return EdaReport {
                symbol: symbol.clone(),
                total_candles: 0,
                start_time: None,
                end_time: None,
                total_duration_days: 0.0,
                invalid_candle_count: 0,
                mathematical_integrity_pct: 100.0,
                min_price: Decimal::ZERO,
                max_price: Decimal::ZERO,
                avg_bar_range_pips: Decimal::ZERO,
                max_single_bar_pips: Decimal::ZERO,
                weekday_gaps_count: 0,
                zero_volume_bars_count: 0,
                health_score: 0.0,
                health_status: "EMPTY DATASET".to_string(),
            };
        }

        let spec = TfPairSpec::from_symbol(symbol);
        let total_candles = candles.len();
        let start_time = Some(candles.first().unwrap().timestamp);
        let end_time = Some(candles.last().unwrap().timestamp);

        let duration_hours = (candles.last().unwrap().timestamp - candles.first().unwrap().timestamp).num_hours();
        let total_duration_days = duration_hours as f64 / 24.0;

        let mut invalid_candle_count = 0;
        let mut min_price = Decimal::MAX;
        let mut max_price = Decimal::MIN;
        let mut total_range_pips = Decimal::ZERO;
        let mut max_single_bar_pips = Decimal::ZERO;
        let mut zero_volume_bars_count = 0;
        let mut weekday_gaps_count = 0;

        for (idx, candle) in candles.iter().enumerate() {
            // 1. Verifikasi Invariant Matematika OHLCV
            let is_valid = candle.high >= candle.low
                && candle.high >= candle.open
                && candle.high >= candle.close
                && candle.low <= candle.open
                && candle.low <= candle.close
                && candle.volume >= Decimal::ZERO;

            if !is_valid {
                invalid_candle_count += 1;
            }

            // 2. Ekstremum Harga
            if candle.low < min_price {
                min_price = candle.low;
            }
            if candle.high > max_price {
                max_price = candle.high;
            }

            // 3. Rentang Volatilitas Bar (High - Low)
            let bar_range_price = candle.high - candle.low;
            let bar_range_pips = spec.price_diff_to_pips(bar_range_price);
            total_range_pips += bar_range_pips;

            if bar_range_pips > max_single_bar_pips {
                max_single_bar_pips = bar_range_pips;
            }

            // 4. Volume Kosong
            if candle.volume.is_zero() {
                zero_volume_bars_count += 1;
            }

            // 5. Deteksi Gap Waktu pada Hari Kerja (Senin - Jumat)
            if idx > 0 {
                let prev_time = candles[idx - 1].timestamp;
                let current_time = candle.timestamp;
                let step_hours = (current_time - prev_time).num_hours();

                // Jika selisih > 1 jam
                if step_hours > 1 {
                    let is_weekend = prev_time.weekday() == chrono::Weekday::Fri
                        || prev_time.weekday() == chrono::Weekday::Sat;
                    if !is_weekend {
                        weekday_gaps_count += 1;
                    }
                }
            }
        }

        let avg_bar_range_pips = if total_candles > 0 {
            total_range_pips / Decimal::from(total_candles)
        } else {
            Decimal::ZERO
        };

        let mathematical_integrity_pct = if total_candles > 0 {
            ((total_candles - invalid_candle_count) as f64 / total_candles as f64) * 100.0
        } else {
            100.0
        };

        // Health Score Calculation (0 to 100)
        let mut health_score = mathematical_integrity_pct;
        if weekday_gaps_count > 0 {
            health_score -= (weekday_gaps_count as f64 * 2.0).min(20.0);
        }
        if zero_volume_bars_count > (total_candles / 10) {
            health_score -= 10.0;
        }

        let health_status = if health_score >= 98.0 {
            "🟢 EXCELLENT (100% Bersih & Siap Riset Multi-Tahun)".to_string()
        } else if health_score >= 80.0 {
            "🟡 GOOD (Data Cukup Baik dengan Sedikit Gap Wajar)".to_string()
        } else {
            "🔴 WARNING (Data Korup / Banyak Anomali)".to_string()
        };

        EdaReport {
            symbol: symbol.clone(),
            total_candles,
            start_time,
            end_time,
            total_duration_days,
            invalid_candle_count,
            mathematical_integrity_pct,
            min_price,
            max_price,
            avg_bar_range_pips,
            max_single_bar_pips,
            weekday_gaps_count,
            zero_volume_bars_count,
            health_score,
            health_status,
        }
    }
}
