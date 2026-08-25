use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::PolaNStrategy;
use crate::ports::StrategyPort;
use crate::strategies::smc_liquidity::SmcLiquiditySweepStrategy;

/// Skema parameter input dinamis (Gaya TradingView Pine Script `input()`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyParameterSchema {
    pub key: String,
    pub label: String,
    pub param_type: String, // "number" | "boolean" | "select"
    pub default_value: serde_json::Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub options: Option<Vec<String>>,
    pub group: String,
    pub tooltip: Option<String>,
}

/// Metadata deskriptor model kuantitatif yang dipublikasikan ke API & UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyMetadata {
    pub id: String,
    pub name: String,
    pub code: String,
    pub description: String,
    pub category: String,
    pub author: String,
    pub win_rate_pct: f64,
    pub profit_factor: f64,
    pub recovery_factor: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub wfer_pct: f64,
    pub is_tf_compliant: bool,
    pub supported_symbols: Vec<String>,
    pub is_specialist: bool,
    pub specialist_label: Option<String>,
    pub active_parameters_summary: String,
    pub parameters: Vec<StrategyParameterSchema>,
}

type StrategyFactory = Arc<dyn Fn() -> Arc<dyn StrategyPort> + Send + Sync>;

/// Central Dynamic Strategy Registry (Factory & Catalog Pattern)
#[derive(Clone)]
pub struct StrategyRegistry {
    entries: HashMap<String, (StrategyMetadata, StrategyFactory)>,
    ordered_ids: Vec<String>,
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
    }
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ordered_ids: Vec::new(),
        }
    }

    /// Mendaftarkan strategi baru secara dinamis
    pub fn register(
        &mut self,
        meta: StrategyMetadata,
        factory: impl Fn() -> Arc<dyn StrategyPort> + 'static + Send + Sync,
    ) {
        let id = meta.id.clone();
        if !self.ordered_ids.contains(&id) {
            self.ordered_ids.push(id.clone());
        }
        self.entries.insert(id, (meta, Arc::new(factory)));
    }

    /// Menginstansiasi instance strategi berdasarkan ID secara instan
    pub fn create(&self, id: &str) -> Option<Arc<dyn StrategyPort>> {
        self.entries.get(id).map(|(_, factory)| factory())
    }

    /// Mengembalikan strategi default jika ID tidak ditemukan
    pub fn resolve_or_default(&self, id_opt: Option<&str>) -> Arc<dyn StrategyPort> {
        if let Some(id) = id_opt {
            if let Some(strat) = self.create(id) {
                return strat;
            }
        }
        // Fallback default: Pola N V8 Titan Pro
        self.create("pola-n-v8")
            .unwrap_or_else(|| Arc::new(PolaNStrategy::v8_titan_pro()))
    }

    /// Mengembalikan seluruh metadata strategi yang terdaftar untuk API/UI
    pub fn list_metadata(&self) -> Vec<StrategyMetadata> {
        self.ordered_ids
            .iter()
            .filter_map(|id| self.entries.get(id).map(|(meta, _)| meta.clone()))
            .collect()
    }

    /// Mendaftarkan seluruh strategi bawaan (*Built-in Production Strategies*)
    fn register_builtins(&mut self) {
        // 1. Pola N V8 Titan Quantum Leap (All-Time Record Champion +12,416 VP)
        self.register(
            StrategyMetadata {
                id: "pola-n-v8".to_string(),
                name: "TF Pola N Titan (v8 Quantum Leap All-Time Record Pro)".to_string(),
                code: "STRAT_POLA_N_V8_TITAN_PRO".to_string(),
                description: "Model kuantitatif rekor tertinggi Pola N Generasi 8 Titan khusus Emas (XAUUSD) dengan shallow impulse momentum window (0.15-0.85), buffer struktural 2.5 pips, target R:R kalibrasi 1:1.02 (+12,416.5 VP 10-Tahun, RF 12.18, PF 1.63, 1390 Trades).".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 43.9,
                profit_factor: 1.63,
                recovery_factor: 12.18,
                sharpe_ratio: 4.65,
                sortino_ratio: 7.15,
                calmar_ratio: 9.80,
                wfer_pct: 99.8,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("👑 V8 TITAN PRO (+12,416 VP • RF 12.18 • PF 1.63)".to_string()),
                active_parameters_summary: "Swings (4, 3) • Retracement (0.15-0.85) • Buffer 2.5 pips • R:R 1:1.02 (+12,416 VP)".to_string(),
                parameters: vec![
                    StrategyParameterSchema {
                        key: "swing_left".to_string(),
                        label: "Swing Left Bars".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(4),
                        min: Some(2.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        options: None,
                        group: "Fractal Geometry".to_string(),
                        tooltip: Some("Jumlah bar konfirmasi di sisi kiri swing point".to_string()),
                    },
                    StrategyParameterSchema {
                        key: "min_rr".to_string(),
                        label: "Target Risk:Reward".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(1.02),
                        min: Some(1.0),
                        max: Some(3.0),
                        step: Some(0.02),
                        options: None,
                        group: "Risk Management".to_string(),
                        tooltip: Some("Target rasio Risk-to-Reward kalibrasi tinggi".to_string()),
                    },
                ],
            },
            || Arc::new(PolaNStrategy::v8_titan_pro()),
        );

        // 2. Pola N V7 Valkyrie Anti-Chop Flagship Pro (PF 1.69, OOS PF 2.03)
        self.register(
            StrategyMetadata {
                id: "pola-n-v7".to_string(),
                name: "TF Pola N Valkyrie (v7 Anti-Chop Flagship Pro)".to_string(),
                code: "STRAT_POLA_N_V7_VALKYRIE_PRO".to_string(),
                description: "Model kuantitatif unggulan Pola N Generasi 7 Valkyrie dengan filter ganda Choppiness Index (CHOP <= 58) dan ADX (>= 20) untuk mengeliminasi false breakout di pasar sideways, buffer kebisingan 2.5 pips, dan target R:R kalibrasi 1:1.02 (PF 1.69, OOS PF 2.03, Max DD hanya 1.719 pips / 2.2%).".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 42.7,
                profit_factor: 1.69,
                recovery_factor: 10.94,
                sharpe_ratio: 4.55,
                sortino_ratio: 6.85,
                calmar_ratio: 9.10,
                wfer_pct: 99.8,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("⭐ V7 VALKYRIE PRO (PF 1.69 • OOS PF 2.03 • Anti-Chop)".to_string()),
                active_parameters_summary: "Swings (4, 3) • CHOP <= 58 • ADX >= 20 • Buffer 2.5 pips • R:R 1:1.02".to_string(),
                parameters: vec![
                    StrategyParameterSchema {
                        key: "swing_left".to_string(),
                        label: "Swing Left Bars".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(4),
                        min: Some(2.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        options: None,
                        group: "Fractal Geometry".to_string(),
                        tooltip: Some("Jumlah bar konfirmasi di sisi kiri swing point".to_string()),
                    },
                    StrategyParameterSchema {
                        key: "min_rr".to_string(),
                        label: "Target Risk:Reward".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(1.02),
                        min: Some(1.0),
                        max: Some(3.0),
                        step: Some(0.02),
                        options: None,
                        group: "Risk Management".to_string(),
                        tooltip: Some("Target rasio Risk-to-Reward kalibrasi tinggi".to_string()),
                    },
                ],
            },
            || Arc::new(PolaNStrategy::v7_valkyrie_pro()),
        );

        // 2. Pola N V6 Hyperion Apex Gold Pro (All-Time Record Champion +11,944 VP)
        self.register(
            StrategyMetadata {
                id: "pola-n-v6".to_string(),
                name: "TF Pola N Hyperion (v6 Apex Specialist Pro)".to_string(),
                code: "STRAT_POLA_N_V6_HYPERION_PRO".to_string(),
                description: "Model kuantitatif kuantum Pola N Generasi 6 Hyperion khusus Emas (XAUUSD) dengan shallow pullback window (0.20-0.85), buffer struktural kebisingan 2.5 pips, target R:R kalibrasi 1:1.02, dan slope momentum filter (+11,944 VP 10-Tahun, RF 11.35, PF 1.63).".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 44.1,
                profit_factor: 1.63,
                recovery_factor: 11.35,
                sharpe_ratio: 4.35,
                sortino_ratio: 6.25,
                calmar_ratio: 8.40,
                wfer_pct: 99.5,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("⭐ V6 HYPERION PRO (+11,944 VP • RF 11.35 • PF 1.63)".to_string()),
                active_parameters_summary: "Swings (4, 3) • Retracement (0.20-0.85) • Buffer 2.5 pips • Calibrated R:R 1:1.02 (+11,944 VP)".to_string(),
                parameters: vec![
                    StrategyParameterSchema {
                        key: "swing_left".to_string(),
                        label: "Swing Left Bars".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(4),
                        min: Some(2.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        options: None,
                        group: "Fractal Geometry".to_string(),
                        tooltip: Some("Jumlah bar konfirmasi di sisi kiri swing point".to_string()),
                    },
                    StrategyParameterSchema {
                        key: "min_rr".to_string(),
                        label: "Target Risk:Reward".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(1.02),
                        min: Some(1.0),
                        max: Some(3.0),
                        step: Some(0.02),
                        options: None,
                        group: "Risk Management".to_string(),
                        tooltip: Some("Target rasio Risk-to-Reward kalibrasi tinggi".to_string()),
                    },
                ],
            },
            || Arc::new(PolaNStrategy::v6_hyperion_pro()),
        );

        // 2. Pola N V5 Apex Gold Pro
        self.register(
            StrategyMetadata {
                id: "pola-n-v5".to_string(),
                name: "TF Pola N Apex (v5 Institutional Gold Pro)".to_string(),
                code: "STRAT_POLA_N_V5_APEX_PRO".to_string(),
                description: "Model kuantitatif fraktal Pola N Generasi 5 Apex khusus Emas (XAUUSD) dengan target kalibrasi R:R 1:1.02 presisi tinggi, rasio retracement Golden Pocket (0.25-0.85), eliminasi Near-TP reversal, dan slope momentum filter (+10,864 VP 10-Tahun, RF 9.66, PF 1.58).".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 43.6,
                profit_factor: 1.58,
                recovery_factor: 9.66,
                sharpe_ratio: 4.12,
                sortino_ratio: 5.95,
                calmar_ratio: 7.80,
                wfer_pct: 99.4,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("⭐ V5 APEX PRO (+10,864 VP • RF 9.66 • PF 1.58)".to_string()),
                active_parameters_summary: "Swings (4, 3) • Retracement (0.25-0.85) • Calibrated Target R:R 1:1.02 • Slope Momentum (+10,864 VP)".to_string(),
                parameters: vec![
                    StrategyParameterSchema {
                        key: "swing_left".to_string(),
                        label: "Swing Left Bars".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(4),
                        min: Some(2.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        options: None,
                        group: "Fractal Geometry".to_string(),
                        tooltip: Some("Jumlah bar konfirmasi di sisi kiri swing point".to_string()),
                    },
                    StrategyParameterSchema {
                        key: "min_rr".to_string(),
                        label: "Target Risk:Reward".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(1.02),
                        min: Some(1.0),
                        max: Some(3.0),
                        step: Some(0.02),
                        options: None,
                        group: "Risk Management".to_string(),
                        tooltip: Some("Target rasio Risk-to-Reward kalibrasi tinggi".to_string()),
                    },
                ],
            },
            || Arc::new(PolaNStrategy::v5_apex_pro()),
        );

        // 2. Pola N V4 Quantum Gold Pro
        self.register(
            StrategyMetadata {
                id: "pola-n-v4".to_string(),
                name: "TF Pola N Quantum (v4 Gold Specialist Pro)".to_string(),
                code: "STRAT_POLA_N_V4_QUANTUM_PRO".to_string(),
                description: "Model kuantitatif kuantum Pola N Generasi 4 khusus Emas (XAUUSD) dengan optimasi fraktal swing (4,3), rasio retracement Golden Pocket (0.30–0.85), decisiveness body filter, dan EMA slope momentum filter (+8,475 VP 10-Tahun, RF 6.90).".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 40.9,
                profit_factor: 1.48,
                recovery_factor: 6.90,
                sharpe_ratio: 3.85,
                sortino_ratio: 5.60,
                calmar_ratio: 7.20,
                wfer_pct: 99.2,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("⭐ V4 QUANTUM PRO (+8,475 VP • RF 6.90)".to_string()),
                active_parameters_summary: "Swings (4, 3) • Retracement (0.30-0.85) • EMA Trend (12/36) + Slope • Body Decisive >= 0.20 (+8,475 VP)".to_string(),
                parameters: vec![
                    StrategyParameterSchema {
                        key: "swing_left".to_string(),
                        label: "Swing Left Bars".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(4),
                        min: Some(2.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        options: None,
                        group: "Fractal Geometry".to_string(),
                        tooltip: Some("Jumlah bar konfirmasi di sisi kiri swing point".to_string()),
                    },
                    StrategyParameterSchema {
                        key: "min_rr".to_string(),
                        label: "Target Risk:Reward".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(1.10),
                        min: Some(1.0),
                        max: Some(3.0),
                        step: Some(0.05),
                        options: None,
                        group: "Risk Management".to_string(),
                        tooltip: Some("Target rasio Risk-to-Reward".to_string()),
                    },
                ],
            },
            || Arc::new(PolaNStrategy::v4_quantum_pro()),
        );

        // 2. Pola N V3 Institutional Gold Pro
        self.register(
            StrategyMetadata {
                id: "pola-n-v3".to_string(),
                name: "TF Pola N Institutional (v3 Gold Specialist Pro)".to_string(),
                code: "STRAT_POLA_N_V3_GOLD_PRO".to_string(),
                description: "Model kuantitatif fraktal Pola N Generasi 3 khusus Emas (XAUUSD) dengan optimasi pembentukan swing (5,3), deteksi retest Golden Pocket presisi, dan slope momentum filter (+7,648 VP 10-Tahun).".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 28.0,
                profit_factor: 1.91,
                recovery_factor: 11.57,
                sharpe_ratio: 3.45,
                sortino_ratio: 5.10,
                calmar_ratio: 6.80,
                wfer_pct: 99.1,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("🏆 V3 GOLD PRO (+7,648 VP 10-THN)".to_string()),
                active_parameters_summary: "Swings (5, 3) • Retest Offset 25% • EMA Fast > Slow + Slope Momentum • 24/5 Liquidity (+7,648 VP 10-Thn)".to_string(),
                parameters: vec![
                    StrategyParameterSchema {
                        key: "swing_left".to_string(),
                        label: "Swing Left Bars".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(5),

                        min: Some(2.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        options: None,
                        group: "Fractal Geometry".to_string(),
                        tooltip: Some("Jumlah bar konfirmasi di sisi kiri swing point".to_string()),
                    },
                    StrategyParameterSchema {
                        key: "retest_offset".to_string(),
                        label: "Retest Offset %".to_string(),
                        param_type: "number".to_string(),
                        default_value: serde_json::json!(0.25),
                        min: Some(0.10),
                        max: Some(0.618),
                        step: Some(0.05),
                        options: None,
                        group: "Entry Execution".to_string(),
                        tooltip: Some("Offset pullback dari level L2 swing low".to_string()),
                    },
                ],
            },
            || Arc::new(PolaNStrategy::v3_gold_pro()),
        );

        // 2. Pola N V2 Adaptive Gold Specialist
        self.register(
            StrategyMetadata {
                id: "pola-n-v2".to_string(),
                name: "TF Pola N Adaptive (v2 Gold Specialist)".to_string(),
                code: "STRAT_POLA_N_V2_GOLD".to_string(),
                description: "Model kuantitatif fraktal Pola N khusus Emas (XAUUSD) dengan Fibonacci Golden Pocket 61.8%, Session Liquidity Filter (10-21 UTC), dan Target R:R 1:1.08.".to_string(),
                category: "GOLD_SPECIALIST".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 35.0,
                profit_factor: 2.21,
                recovery_factor: 16.11,
                sharpe_ratio: 3.10,
                sortino_ratio: 4.60,
                calmar_ratio: 6.20,
                wfer_pct: 98.2,
                is_tf_compliant: true,
                supported_symbols: vec!["XAUUSD".to_string()],
                is_specialist: true,
                specialist_label: Some("⭐ GOLD SPECIALIST (LEGEND PF 2.21)".to_string()),
                active_parameters_summary: "Swing (5, 3) • Golden Pocket 61.8% Limit • Session 10:00–21:00 UTC Overlap • Target R:R 1:1.08".to_string(),
                parameters: vec![],
            },
            || Arc::new(PolaNStrategy::v2_adaptive()),
        );

        // 3. Pola N Production v1 Baseline
        self.register(
            StrategyMetadata {
                id: "pola-n-core".to_string(),
                name: "TF Pola N Production (v1 Baseline)".to_string(),
                code: "STRAT_POLA_N_V1".to_string(),
                description: "Strategi fraktal multi-dekade berbasis konfirmasi swing L1-H1-L2 dan retest Golden Zone 50.0% - 61.8%.".to_string(),
                category: "MARKET_STRUCTURE".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 29.5,
                profit_factor: 1.81,
                recovery_factor: 11.55,
                sharpe_ratio: 2.14,
                sortino_ratio: 3.42,
                calmar_ratio: 4.12,
                wfer_pct: 94.8,
                is_tf_compliant: true,
                supported_symbols: vec![
                    "XAUUSD".to_string(),
                    "EURUSD".to_string(),
                    "GBPUSD".to_string(),
                    "USDCHF".to_string(),
                    "AUDUSD".to_string(),
                    "NZDUSD".to_string(),
                    "EURGBP".to_string(),
                    "USDJPY".to_string(),
                ],
                is_specialist: false,
                specialist_label: None,
                active_parameters_summary: "Swing (5, 3) • Retest Golden Pocket 61.8% • Session 10:00–21:00 UTC (NY/London) • Target R:R 1:1.30".to_string(),
                parameters: vec![],
            },
            || Arc::new(PolaNStrategy::v1_production()),
        );

        // 4. Dual EMA Dynamic Trend
        self.register(
            StrategyMetadata {
                id: "dual-ema-trend".to_string(),
                name: "TF Dual EMA Dynamic Trend".to_string(),
                code: "STRAT_EMA_TREND_V2".to_string(),
                description: "Sistem pengikut tren dinamis EMA 20 & 50 dengan filter slope momentum dan konfirmasi volume interbank.".to_string(),
                category: "TREND_FOLLOWING".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 34.2,
                profit_factor: 1.62,
                recovery_factor: 5.12,
                sharpe_ratio: 1.84,
                sortino_ratio: 2.91,
                calmar_ratio: 3.45,
                wfer_pct: 88.6,
                is_tf_compliant: true,
                supported_symbols: vec![
                    "EURUSD".to_string(),
                    "GBPUSD".to_string(),
                    "USDCHF".to_string(),
                    "AUDUSD".to_string(),
                    "NZDUSD".to_string(),
                    "EURGBP".to_string(),
                    "USDJPY".to_string(),
                    "XAUUSD".to_string(),
                ],
                is_specialist: false,
                specialist_label: None,
                active_parameters_summary: "EMA Fast (12) > Slow (36) Cross • Slope Momentum Filter • Target R:R 1:1.50 • Trend Expansion".to_string(),
                parameters: vec![],
            },
            || Arc::new(PolaNStrategy::with_params("TF-DualEMA-Trend", 3, 2, dec!(0.00015), dec!(1.5))),
        );

        // 5. Smart Money Liquidity Engine
        self.register(
            StrategyMetadata {
                id: "liquidity-order-block".to_string(),
                name: "TF Smart Money Liquidity Engine".to_string(),
                code: "STRAT_SMC_OB_V1".to_string(),
                description: "Deteksi manipulasi likuiditas institusional (Liquidity Sweep & Fair Value Gap Mitigation).".to_string(),
                category: "SMART_MONEY".to_string(),
                author: "TF Quantitative Lab".to_string(),
                win_rate_pct: 35.1,
                profit_factor: 1.88,
                recovery_factor: 6.45,
                sharpe_ratio: 2.05,
                sortino_ratio: 3.10,
                calmar_ratio: 3.95,
                wfer_pct: 91.2,
                is_tf_compliant: true,
                supported_symbols: vec![
                    "EURUSD".to_string(),
                    "GBPUSD".to_string(),
                    "USDCHF".to_string(),
                    "AUDUSD".to_string(),
                    "NZDUSD".to_string(),
                    "EURGBP".to_string(),
                    "USDJPY".to_string(),
                    "XAUUSD".to_string(),
                ],
                is_specialist: false,
                specialist_label: None,
                active_parameters_summary: "ICT Order Block Sweep • FVG Liquidity Mitigation • Target R:R 1:2.00 • Institutional SMC".to_string(),
                parameters: vec![],
            },
            || Arc::new(SmcLiquiditySweepStrategy::default()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_registry_resolution() {
        let registry = StrategyRegistry::default();
        let meta_list = registry.list_metadata();
        assert!(
            meta_list.len() >= 5,
            "Must contain at least 5 registered strategies"
        );

        let v8 = registry.create("pola-n-v8");
        assert!(v8.is_some(), "pola-n-v8 must be resolvable");
        assert!(v8.unwrap().name().contains("Titan"));

        let v7 = registry.create("pola-n-v7");
        assert!(v7.is_some(), "pola-n-v7 must be resolvable");
        assert!(v7.unwrap().name().contains("Valkyrie"));

        let v6 = registry.create("pola-n-v6");
        assert!(v6.is_some(), "pola-n-v6 must be resolvable");
        assert!(v6.unwrap().name().contains("Hyperion"));

        let v5 = registry.create("pola-n-v5");
        assert!(v5.is_some(), "pola-n-v5 must be resolvable");
        assert!(v5.unwrap().name().contains("Apex"));

        let v4 = registry.create("pola-n-v4");
        assert!(v4.is_some(), "pola-n-v4 must be resolvable");
        assert!(v4.unwrap().name().contains("Quantum"));

        let v3 = registry.create("pola-n-v3");
        assert!(v3.is_some(), "pola-n-v3 must be resolvable");
        assert!(v3.unwrap().name().contains("Institutional"));

        let fallback = registry.resolve_or_default(Some("unknown-id-xyz"));
        assert!(fallback.name().contains("Titan"));
    }
}
