use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::info;

use domain::errors::DomainError;
use domain::models::{BacktestConfig, RiskProfile, Symbol, Timeframe};
use domain::ports::{MarketDataPort, StrategyPort};

use super::backtest::BacktestService;

/// Entri Peringkat pada Leaderboard Turnamen Strategi
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyLeaderboardEntry {
    pub rank: usize,
    pub strategy_name: String,
    pub total_trades: usize,
    pub win_rate_percent: Decimal,
    pub total_raw_pips: Decimal,
    pub total_valued_pips: Decimal,
    pub profit_factor: Decimal,
    pub recovery_factor: Decimal,
    pub monthly_loss_ratio_percent: Decimal,
    pub tf_status: String,
}

/// Service Turnamen & Komparasi Multi-Strategi
pub struct StrategyBenchmarkService {
    market_data: Arc<dyn MarketDataPort>,
    risk_profile: RiskProfile,
    config: BacktestConfig,
}

impl StrategyBenchmarkService {
    pub fn new(
        market_data: Arc<dyn MarketDataPort>,
        risk_profile: RiskProfile,
        config: BacktestConfig,
    ) -> Self {
        Self {
            market_data,
            risk_profile,
            config,
        }
    }

    /// Menjalankan turnamen kompetisi multi-strategi pada dataset pasar yang sama
    pub async fn run_tournament(
        &self,
        strategies: &[Arc<dyn StrategyPort>],
        symbols: &[Symbol],
        timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StrategyLeaderboardEntry>, DomainError> {
        info!(
            "Memulai Strategy Tournament untuk {} strategi pada {} pair...",
            strategies.len(),
            symbols.len()
        );

        let mut entries = Vec::new();

        for strategy in strategies {
            let backtester = BacktestService::with_config(
                self.market_data.clone(),
                strategy.clone(),
                self.risk_profile.clone(),
                self.config.clone(),
            );

            let mut strat_total_trades = 0;
            let mut strat_winning_trades = 0;
            let mut strat_raw_pips = Decimal::ZERO;
            let mut strat_valued_pips = Decimal::ZERO;
            let mut strat_gross_profit = Decimal::ZERO;
            let mut strat_gross_loss = Decimal::ZERO;
            let mut strat_max_dd = Decimal::ZERO;

            for symbol in symbols {
                let report = backtester
                    .run_simulation(symbol, timeframe, from, to)
                    .await?;

                strat_total_trades += report.total_trades;
                strat_winning_trades += report.winning_trades;
                strat_raw_pips += report.total_raw_pips;
                strat_valued_pips += report.total_valued_pips;
                strat_gross_profit += report.gross_profit_pips;
                strat_gross_loss += report.gross_loss_pips;
                if report.max_drawdown_pips > strat_max_dd {
                    strat_max_dd = report.max_drawdown_pips;
                }
            }

            let win_rate = if strat_total_trades > 0 {
                (Decimal::from(strat_winning_trades) / Decimal::from(strat_total_trades))
                    * rust_decimal_macros::dec!(100.0)
            } else {
                Decimal::ZERO
            };

            let pf = if strat_gross_loss > Decimal::ZERO {
                strat_gross_profit / strat_gross_loss
            } else if strat_gross_profit > Decimal::ZERO {
                rust_decimal_macros::dec!(99.99)
            } else {
                Decimal::ZERO
            };

            let rec_factor = if strat_max_dd > Decimal::ZERO {
                strat_raw_pips / strat_max_dd
            } else {
                Decimal::ZERO
            };

            let loss_ratio = if strat_gross_profit > Decimal::ZERO {
                (strat_gross_loss / strat_gross_profit) * rust_decimal_macros::dec!(100.0)
            } else {
                Decimal::ZERO
            };

            let tf_status = if strat_valued_pips >= rust_decimal_macros::dec!(300.0)
                && strat_total_trades >= 5
                && pf >= rust_decimal_macros::dec!(2.10)
            {
                "🌟 LEGEND QUALIFIED"
            } else if strat_valued_pips >= rust_decimal_macros::dec!(300.0) && strat_total_trades >= 5 {
                "🌟 MASTER QUALIFIED"
            } else {
                "⚠️ REVIEW NEEDED"
            };

            entries.push(StrategyLeaderboardEntry {
                rank: 0,
                strategy_name: strategy.name().to_string(),
                total_trades: strat_total_trades,
                win_rate_percent: win_rate,
                total_raw_pips: strat_raw_pips,
                total_valued_pips: strat_valued_pips,
                profit_factor: pf,
                recovery_factor: rec_factor,
                monthly_loss_ratio_percent: loss_ratio,
                tf_status: tf_status.to_string(),
            });
        }

        // Urutkan berdasarkan total Valued Pips tertinggi
        entries.sort_by_key(|e| std::cmp::Reverse(e.total_valued_pips));


        for (idx, entry) in entries.iter_mut().enumerate() {
            entry.rank = idx + 1;
        }

        Ok(entries)
    }
}
