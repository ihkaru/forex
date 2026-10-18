use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::PolaNStrategy;
use crate::ports::StrategyPort;

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
        // Fallback default: Pola N V9 Titan Ultra
        self.create("pola-n-v9")
            .unwrap_or_else(|| Arc::new(PolaNStrategy::v9_titan_ultra()))
    }

    /// Mengembalikan seluruh metadata strategi yang terdaftar untuk API/UI
    pub fn list_metadata(&self) -> Vec<StrategyMetadata> {
        self.ordered_ids
            .iter()
            .filter_map(|id| self.entries.get(id).map(|(meta, _)| meta.clone()))
            .collect()
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
