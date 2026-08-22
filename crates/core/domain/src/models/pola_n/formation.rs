use super::detector::SwingPoint;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Tipe Formasi Pola N
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolaNType {
    BullishN,
    BearishN,
}

/// Struktur Hasil Validasi Formasi Pola N
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolaNFormation {
    pub pattern_type: PolaNType,
    pub point_1: Decimal, // Start (L1 or H1)
    pub point_2: Decimal, // Peak / Trough (H1 or L1)
    pub point_3: Decimal, // Retest (L2 Higher Low or H2 Lower High)
    pub suggested_entry: Decimal,
    pub stop_loss: Decimal,
    pub take_profit_1: Decimal,
    pub take_profit_2: Decimal,
    pub risk_reward_ratio: Decimal,
}

/// Komponen 2: Engine Pengenal Formasi Pola N (Traders Family Standard)
#[derive(Debug, Clone)]
pub struct PolaNFormationEngine {
    pub pip_buffer: Decimal,
    pub min_rr_ratio: Decimal,
}

impl Default for PolaNFormationEngine {
    fn default() -> Self {
        Self {
            pip_buffer: dec!(0.00020), // 2.0 pips buffer untuk 5-digit pair
            min_rr_ratio: dec!(1.49),  // User requested R:R 1:1.49
        }
    }
}

impl PolaNFormationEngine {
    pub fn new(pip_buffer: Decimal, min_rr_ratio: Decimal) -> Self {
        Self {
            pip_buffer,
            min_rr_ratio,
        }
    }

    /// Evaluasi apakah 3 titik swing berurutan membentuk Pola N yang valid.
    /// `pip_size` dipakai untuk kalkulasi SL struktural presisi berbasis L2/H2.
    pub fn evaluate_swings(
        &self,
        swings: &[SwingPoint],
        _current_price: Decimal,
        pip_size: Decimal,
    ) -> Option<PolaNFormation> {
        if swings.len() < 3 {
            return None;
        }

        let n = swings.len();
        let p1 = &swings[n - 3];
        let p2 = &swings[n - 2];
        let p3 = &swings[n - 1];

        // 1. EVALUASI POLA N BULLISH: (Low1 -> High1 -> HigherLow2)
        if !p1.is_high && p2.is_high && !p3.is_high {
            let l1 = p1.price;
            let h1 = p2.price;
            let l2 = p3.price;

            // Syarat Geometris Pola N: L1 < L2 < H1 (Higher Low & Valid Impulsive Expansion)
            if l1 < l2 && l2 < h1 {
                let impulse = h1 - l1;
                let retracement_ratio = (h1 - l2) / impulse;

                // Pullback di area Fibonacci retracement yang valid (38% – 85%)
                // 38% = shallow pullback (strong trend), 85% = deep discount (golden pocket)
                if retracement_ratio >= dec!(0.38) && retracement_ratio <= dec!(0.85) {
                    let pullback_range = h1 - l2;
                    // Golden Pullback Entry: Entry Limit di 25% di atas swing low L2
                    // Keunggulan matematis: SL sangat ketat di bawah L2 (10-18 pips),
                    // sehingga TP1 (RR 1:1.5) sudah tercapai SEBELUM harga menyentuh H1!
                    let entry = l2 + (pullback_range * dec!(0.25));
                    // SL Struktural: Tepat di bawah Higher Low (L2) dengan 2 pips buffer
                    let sl = l2 - (pip_size * dec!(2.0));
                    let risk_distance = entry - sl;

                    if risk_distance > Decimal::ZERO {
                        let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));
                        let tp1 = entry + (risk_distance * target_rr);
                        let reward_distance = tp1 - entry;

                        if reward_distance > Decimal::ZERO {
                            let rr = (reward_distance / risk_distance).clamp(dec!(1.0), dec!(3.0));
                            let tp2 = entry + (risk_distance * dec!(2.8));

                            return Some(PolaNFormation {
                                pattern_type: PolaNType::BullishN,
                                point_1: l1,
                                point_2: h1,
                                point_3: l2,
                                suggested_entry: entry,
                                stop_loss: sl,
                                take_profit_1: tp1,
                                take_profit_2: tp2,
                                risk_reward_ratio: rr,
                            });
                        }
                    }
                }
            }
        }

        // 2. EVALUASI POLA N BEARISH (N-Terbalik): (High1 -> Low1 -> LowerHigh2)
        if p1.is_high && !p2.is_high && p3.is_high {
            let h1 = p1.price;
            let l1 = p2.price;
            let h2 = p3.price;

            // Syarat Geometris Pola N Terbalik: L1 < H2 < H1 (Lower High & Valid Breakdown)
            if l1 < h2 && h2 < h1 {
                let impulse = h1 - l1;
                let retracement_ratio = (h2 - l1) / impulse;

                // Pullback di area Fibonacci retracement yang valid (38% – 85%)
                if retracement_ratio >= dec!(0.38) && retracement_ratio <= dec!(0.85) {
                    let pullback_range = h2 - l1;
                    // Golden Pullback Entry: Entry Limit di 25% di bawah swing high H2
                    let entry = h2 - (pullback_range * dec!(0.25));
                    // SL Struktural: Tepat di atas Lower High (H2) dengan 2 pips buffer
                    let sl = h2 + (pip_size * dec!(2.0));
                    let risk_distance = sl - entry;

                    if risk_distance > Decimal::ZERO {
                        let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));
                        let tp1 = entry - (risk_distance * target_rr);
                        let reward_distance = entry - tp1;

                        if reward_distance > Decimal::ZERO {
                            let rr = (reward_distance / risk_distance).clamp(dec!(1.0), dec!(3.0));
                            let tp2 = entry - (risk_distance * dec!(2.8));

                            return Some(PolaNFormation {
                                pattern_type: PolaNType::BearishN,
                                point_1: h1,
                                point_2: l1,
                                point_3: h2,
                                suggested_entry: entry,
                                stop_loss: sl,
                                take_profit_1: tp1,
                                take_profit_2: tp2,
                                risk_reward_ratio: rr,
                            });
                        }
                    }
                }
            }
        }

        None
    }
}
