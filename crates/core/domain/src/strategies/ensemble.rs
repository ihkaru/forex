use async_trait::async_trait;
use std::sync::Arc;

use crate::errors::DomainError;
use crate::models::Signal;
use crate::ports::{MarketContext, StrategyPort};

/// Strategi Komposit (Ensemble / Multi-Strategy Confirmation)
pub struct EnsembleStrategy {
    pub name: String,
    pub primary: Arc<dyn StrategyPort>,
    pub secondary: Arc<dyn StrategyPort>,
}

impl EnsembleStrategy {
    pub fn new(
        name: impl Into<String>,
        primary: Arc<dyn StrategyPort>,
        secondary: Arc<dyn StrategyPort>,
    ) -> Self {
        Self {
            name: name.into(),
            primary,
            secondary,
        }
    }
}

#[async_trait]
impl StrategyPort for EnsembleStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError> {
        let sig1 = self.primary.evaluate(ctx).await?;
        let sig2 = self.secondary.evaluate(ctx).await?;

        // Jika kedua strategi menghasilkan sinyal pada arah yang sama (High Confidence Confluence)
        if let (Some(mut s1), Some(s2)) = (sig1, sig2) {
            if s1.action == s2.action {
                s1.confidence_score = 0.98;
                s1.strategy_name = self.name.clone();
                s1.rationale = format!(
                    "Hybrid Confluence [{} + {}]: {}",
                    self.primary.name(),
                    self.secondary.name(),
                    s1.rationale
                );
                return Ok(Some(s1));
            }
        }

        Ok(None)
    }
}
