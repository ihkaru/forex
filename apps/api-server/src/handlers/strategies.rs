use axum::Json;
use domain::strategies::{StrategyMetadata, StrategyRegistry};

/// Handler REST API: Mengembalikan daftar seluruh metadata model kuantitatif yang terdaftar di Registry
pub async fn strategies_handler() -> Json<Vec<StrategyMetadata>> {
    let registry = StrategyRegistry::default();
    Json(registry.list_metadata())
}
