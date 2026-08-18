use async_trait::async_trait;
use crate::errors::DomainError;
use crate::models::{Candle, RiskProfile, Signal, Symbol, Tick, Timeframe};

pub struct MarketContext<'a> {
    pub symbol: &'a Symbol,
    pub timeframe: Timeframe,
    pub current_tick: &'a Tick,
    pub candles: &'a [Candle],
    pub risk_profile: &'a RiskProfile,
}

#[async_trait]
pub trait StrategyPort: Send + Sync {
    /// Nama unik strategi (contoh: "SMC-Liquidity-Sweep-v1")
    fn name(&self) -> &str;

    /// Evaluasi kondisi pasar saat ini dan hasilkan sinyal jika kondisi terpenuhi
    async fn evaluate(&self, ctx: &MarketContext<'_>) -> Result<Option<Signal>, DomainError>;
}
