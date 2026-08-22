use crate::models::Candle;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Titik Swing High / Swing Low Fraktal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwingPoint {
    pub index: usize,
    pub price: Decimal,
    pub is_high: bool,
}

/// Komponen 1: Detektor Fraktal Geometri Pasar (Decoupled Swing Point Detector)
#[derive(Debug, Clone)]
pub struct SwingPointDetector {
    pub left_bars: usize,
    pub right_bars: usize,
}

impl Default for SwingPointDetector {
    fn default() -> Self {
        Self {
            left_bars: 5,
            right_bars: 3,
        }
    }
}

impl SwingPointDetector {
    pub fn new(left_bars: usize, right_bars: usize) -> Self {
        Self {
            left_bars,
            right_bars,
        }
    }

    /// Mendeteksi seluruh titik swing high dan swing low secara bar-by-bar
    pub fn detect_swings(&self, candles: &[Candle]) -> Vec<SwingPoint> {
        let mut swings = Vec::new();
        let total = candles.len();

        if total < self.left_bars + self.right_bars + 1 {
            return swings;
        }

        for i in self.left_bars..(total - self.right_bars) {
            let current_high = candles[i].high;
            let current_low = candles[i].low;

            // Cek Swing High
            let is_swing_high = (1..=self.left_bars).all(|l| candles[i - l].high < current_high)
                && (1..=self.right_bars).all(|r| candles[i + r].high < current_high);

            if is_swing_high {
                swings.push(SwingPoint {
                    index: i,
                    price: current_high,
                    is_high: true,
                });
                continue;
            }

            // Cek Swing Low
            let is_swing_low = (1..=self.left_bars).all(|l| candles[i - l].low > current_low)
                && (1..=self.right_bars).all(|r| candles[i + r].low > current_low);

            if is_swing_low {
                swings.push(SwingPoint {
                    index: i,
                    price: current_low,
                    is_high: false,
                });
            }
        }

        swings
    }
}

/// Utilitas Perhitungan Exponential Moving Average (EMA)
pub fn calculate_ema(candles: &[Candle], period: usize) -> Option<Decimal> {
    if candles.len() < period || period == 0 {
        return None;
    }
    let k = Decimal::from(2) / Decimal::from(period + 1);
    let initial_sma: Decimal =
        candles[..period].iter().map(|c| c.close).sum::<Decimal>() / Decimal::from(period);
    let mut ema = initial_sma;
    for candle in &candles[period..] {
        ema = (candle.close * k) + (ema * (Decimal::ONE - k));
    }
    Some(ema)
}

/// Utilitas Perhitungan Average True Range (ATR)
pub fn calculate_atr(candles: &[Candle], period: usize) -> Option<Decimal> {
    if candles.len() < period + 1 || period == 0 {
        return None;
    }
    let mut tr_sum = Decimal::ZERO;
    for i in (candles.len() - period)..candles.len() {
        let h = candles[i].high;
        let l = candles[i].low;
        let prev_c = candles[i - 1].close;
        let tr1 = h - l;
        let tr2 = (h - prev_c).abs();
        let tr3 = (l - prev_c).abs();
        let tr = tr1.max(tr2).max(tr3);
        tr_sum += tr;
    }
    Some(tr_sum / Decimal::from(period))
}

/// Utilitas Perhitungan Relative Strength Index (RSI) - 100% Decimal Murni
pub fn calculate_rsi(candles: &[Candle], period: usize) -> Option<Decimal> {
    if candles.len() < period + 1 || period == 0 {
        return None;
    }
    let mut gains = Decimal::ZERO;
    let mut losses = Decimal::ZERO;

    for i in (candles.len() - period)..candles.len() {
        let diff = candles[i].close - candles[i - 1].close;
        if diff > Decimal::ZERO {
            gains += diff;
        } else {
            losses += diff.abs();
        }
    }

    if losses.is_zero() {
        return Some(dec!(100.0));
    }
    if gains.is_zero() {
        return Some(Decimal::ZERO);
    }

    let rs = gains / losses;
    let rsi = dec!(100.0) - (dec!(100.0) / (Decimal::ONE + rs));
    Some(rsi)
}

/// Utilitas Perhitungan Kemiringan Trend (Slope) EMA
pub fn calculate_ema_slope(candles: &[Candle], period: usize, lookback: usize) -> Option<Decimal> {
    if candles.len() < period + lookback {
        return None;
    }
    let ema_now = calculate_ema(candles, period)?;
    let ema_prev = calculate_ema(&candles[..candles.len() - lookback], period)?;
    Some(ema_now - ema_prev)
}
