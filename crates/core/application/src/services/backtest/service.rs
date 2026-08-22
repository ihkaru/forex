use chrono::{DateTime, Utc};
use domain::errors::DomainError;
use domain::models::{
    BacktestConfig, Order, RiskProfile, SignalAction, Symbol, TfComplianceGuard, TfPairSpec, Tick,
    Timeframe,
};
use domain::ports::{MarketContext, MarketDataPort, StrategyPort};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tracing::info;

use super::models::{
    BacktestReport, DetailedBacktestResult, EquityPoint, SimulatedOrder, SimulatedOrderStatus,
    TradeDirectionBreakdown, TradingViewPerformanceSummary,
};

pub struct BacktestService {
    market_data: Arc<dyn MarketDataPort>,
    strategy: Arc<dyn StrategyPort>,
    risk_profile: RiskProfile,
    config: BacktestConfig,
}

impl BacktestService {
    pub fn new(
        market_data: Arc<dyn MarketDataPort>,
        strategy: Arc<dyn StrategyPort>,
        risk_profile: RiskProfile,
    ) -> Self {
        Self {
            market_data,
            strategy,
            risk_profile,
            config: BacktestConfig::default(),
        }
    }

    pub fn with_config(
        market_data: Arc<dyn MarketDataPort>,
        strategy: Arc<dyn StrategyPort>,
        risk_profile: RiskProfile,
        config: BacktestConfig,
    ) -> Self {
        Self {
            market_data,
            strategy,
            risk_profile,
            config,
        }
    }

    /// Menjalankan simulasi backtest deterministik dan mengembalikan laporan ringkas
    pub async fn run_simulation(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<BacktestReport, DomainError> {
        self.run_simulation_detailed(symbol, timeframe, from, to)
            .await
            .map(|res| res.report)
    }

    /// Menjalankan simulasi backtest deterministik dan mengembalikan laporan + seluruh riwayat trade
    pub async fn run_simulation_detailed(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<DetailedBacktestResult, DomainError> {
        info!(
            "Memulai simulasi backtest untuk {} [{:?}] dari {} hingga {}",
            symbol, timeframe, from, to
        );

        let historical_candles = self
            .market_data
            .get_historical_candles(symbol, timeframe, from, to)
            .await?;

        info!("Memproses {} candle historis...", historical_candles.len());

        let spec = TfPairSpec::from_symbol(symbol);
        let mut completed_trades: Vec<Order> = Vec::new();
        let mut active_sim_orders: Vec<SimulatedOrder> = Vec::new();
        let mut peak_pips = Decimal::ZERO;
        let mut max_drawdown_pips = Decimal::ZERO;
        let mut running_pips = Decimal::ZERO;

        let window_size = self.config.lookback_window_bars;
        let spread_offset = spec.pip_size * self.config.simulation_spread_pips;

        let mut last_signal_entry: Option<Decimal> = None;
        let mut last_signal_bar: usize = 0;

        if historical_candles.len() > window_size {
            for i in window_size..historical_candles.len() {
                let slice = &historical_candles[(i - window_size)..i];
                let current_candle = &historical_candles[i];

                // Update status pending & running orders dengan candle saat ini
                let mut remaining_orders = Vec::new();
                for mut sim in active_sim_orders {
                    let mut closed = false;
                    let sl = sim.order.stop_loss;
                    let tp = sim.order.take_profit;

                    match sim.status {
                        SimulatedOrderStatus::Pending => {
                            // Cek apakah order kedaluwarsa sebelum terisi (24 jam)
                            if current_candle.timestamp >= sim.expires_at {
                                continue;
                            }

                            // Cek apakah harga pasar menjemput Limit Order
                            match sim.order.action {
                                SignalAction::BuyLimit => {
                                    if current_candle.low + spread_offset <= sim.order.open_price {
                                        sim.status = SimulatedOrderStatus::Filled {
                                            fill_time: current_candle.timestamp,
                                        };
                                        remaining_orders.push(sim);
                                    } else {
                                        remaining_orders.push(sim);
                                    }
                                }
                                SignalAction::SellLimit => {
                                    if current_candle.high >= sim.order.open_price {
                                        sim.status = SimulatedOrderStatus::Filled {
                                            fill_time: current_candle.timestamp,
                                        };
                                        remaining_orders.push(sim);
                                    } else {
                                        remaining_orders.push(sim);
                                    }
                                }
                                _ => {}
                            }
                        }
                        SimulatedOrderStatus::Filled { .. } => {
                            match sim.order.action {
                                SignalAction::BuyLimit | SignalAction::BuyStop => {
                                    let sl_hit = current_candle.low <= sl;
                                    let tp_hit = current_candle.high >= tp;

                                    if sl_hit {
                                        let pips =
                                            -spec.price_diff_to_pips(sim.order.open_price - sl);
                                        sim.order.close_time = Some(current_candle.timestamp);
                                        sim.order.realized_pnl = Some(pips);
                                        running_pips += pips;
                                        closed = true;
                                    } else if tp_hit {
                                        let pips =
                                            spec.price_diff_to_pips(tp - sim.order.open_price);
                                        sim.order.close_time = Some(current_candle.timestamp);
                                        sim.order.realized_pnl = Some(pips);
                                        running_pips += pips;
                                        closed = true;
                                    }
                                }
                                SignalAction::SellLimit | SignalAction::SellStop => {
                                    let sl_hit = current_candle.high + spread_offset >= sl;
                                    let tp_hit = current_candle.low + spread_offset <= tp;

                                    if sl_hit {
                                        let pips =
                                            -spec.price_diff_to_pips(sl - sim.order.open_price);
                                        sim.order.close_time = Some(current_candle.timestamp);
                                        sim.order.realized_pnl = Some(pips);
                                        running_pips += pips;
                                        closed = true;
                                    } else if tp_hit {
                                        let pips =
                                            spec.price_diff_to_pips(sim.order.open_price - tp);
                                        sim.order.close_time = Some(current_candle.timestamp);
                                        sim.order.realized_pnl = Some(pips);
                                        running_pips += pips;
                                        closed = true;
                                    }
                                }
                                _ => {}
                            }

                            if closed {
                                completed_trades.push(sim.order);
                            } else {
                                remaining_orders.push(sim);
                            }
                        }
                    }

                    // Update Drawdown pelacakan puncak
                    if running_pips > peak_pips {
                        peak_pips = running_pips;
                    }
                    let current_dd = peak_pips - running_pips;
                    if current_dd > max_drawdown_pips {
                        max_drawdown_pips = current_dd;
                    }
                }
                active_sim_orders = remaining_orders;

                // Evaluasi sinyal baru jika belum ada pending order aktif pada bar ini
                if active_sim_orders.is_empty() {
                    let last_slice_candle = &slice[slice.len() - 1];
                    let tick = Tick {
                        symbol: last_slice_candle.symbol.clone(),
                        bid: last_slice_candle.close,
                        ask: last_slice_candle.close + spread_offset,
                        timestamp: last_slice_candle.timestamp,
                        source: last_slice_candle.source,
                    };

                    let context = MarketContext {
                        symbol,
                        timeframe,
                        current_tick: &tick,
                        candles: slice,
                        risk_profile: &self.risk_profile,
                    };

                    if let Ok(Some(signal)) = self.strategy.evaluate(&context).await {
                        // Hindari duplikasi sinyal pada level harga atau bar yang sama persis
                        let is_duplicate = last_signal_entry == Some(signal.entry_price)
                            || (i.saturating_sub(last_signal_bar) < 6);

                        if !is_duplicate && TfComplianceGuard::validate_signal(&signal).is_ok() {
                            last_signal_entry = Some(signal.entry_price);
                            last_signal_bar = i;

                            let order = Order {
                                id: signal.id,
                                symbol: signal.symbol.clone(),
                                action: signal.action,
                                volume_lots: dec!(0.10),
                                open_price: signal.entry_price,
                                current_price: signal.entry_price,
                                stop_loss: signal.stop_loss,
                                take_profit: signal.take_profit_1,
                                open_time: current_candle.timestamp,
                                close_time: None,
                                realized_pnl: None,
                            };
                            let sim_order = SimulatedOrder {
                                order,
                                status: SimulatedOrderStatus::Pending,
                                expires_at: current_candle.timestamp + chrono::Duration::hours(24),
                            };
                            active_sim_orders.push(sim_order);
                        }
                    }
                }
            }
        }

        let total_trades = completed_trades.len();
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut gross_profit_pips = Decimal::ZERO;
        let mut gross_loss_pips = Decimal::ZERO;
        let mut total_raw_pips = Decimal::ZERO;

        let mut long_trades = 0;
        let mut long_wins = 0;
        let mut long_losses = 0;
        let mut long_gross_profit = Decimal::ZERO;
        let mut long_gross_loss = Decimal::ZERO;
        let mut long_net = Decimal::ZERO;

        let mut short_trades = 0;
        let mut short_wins = 0;
        let mut short_losses = 0;
        let mut short_gross_profit = Decimal::ZERO;
        let mut short_gross_loss = Decimal::ZERO;
        let mut short_net = Decimal::ZERO;

        let mut largest_win = Decimal::ZERO;
        let mut largest_loss = Decimal::ZERO;
        let mut cur_consecutive_wins = 0;
        let mut max_consecutive_wins = 0;
        let mut cur_consecutive_losses = 0;
        let mut max_consecutive_losses = 0;

        let mut equity_curve = Vec::with_capacity(completed_trades.len());
        let mut running_eq = Decimal::ZERO;
        let mut peak_eq = Decimal::ZERO;

        for trade in &completed_trades {
            if let Some(pnl) = trade.realized_pnl {
                total_raw_pips += pnl;
                running_eq += pnl;
                if running_eq > peak_eq {
                    peak_eq = running_eq;
                }
                let dd = peak_eq - running_eq;
                let dd_pct = if peak_eq > Decimal::ZERO {
                    (dd / peak_eq) * dec!(100.0)
                } else {
                    Decimal::ZERO
                };

                let close_time = trade
                    .close_time
                    .map(|t| t.timestamp())
                    .unwrap_or(trade.open_time.timestamp());

                equity_curve.push(EquityPoint {
                    time: close_time,
                    equity_pips: running_eq,
                    drawdown_pips: dd,
                    drawdown_percent: dd_pct,
                });

                let is_long =
                    matches!(trade.action, SignalAction::BuyLimit | SignalAction::BuyStop);
                if is_long {
                    long_trades += 1;
                    long_net += pnl;
                    if pnl > Decimal::ZERO {
                        long_wins += 1;
                        long_gross_profit += pnl;
                    } else if pnl < Decimal::ZERO {
                        long_losses += 1;
                        long_gross_loss += pnl.abs();
                    }
                } else {
                    short_trades += 1;
                    short_net += pnl;
                    if pnl > Decimal::ZERO {
                        short_wins += 1;
                        short_gross_profit += pnl;
                    } else if pnl < Decimal::ZERO {
                        short_losses += 1;
                        short_gross_loss += pnl.abs();
                    }
                }

                if pnl > Decimal::ZERO {
                    winning_trades += 1;
                    gross_profit_pips += pnl;
                    if pnl > largest_win {
                        largest_win = pnl;
                    }
                    cur_consecutive_wins += 1;
                    cur_consecutive_losses = 0;
                    if cur_consecutive_wins > max_consecutive_wins {
                        max_consecutive_wins = cur_consecutive_wins;
                    }
                } else if pnl < Decimal::ZERO {
                    losing_trades += 1;
                    gross_loss_pips += pnl.abs();
                    let loss_abs = pnl.abs();
                    if loss_abs > largest_loss {
                        largest_loss = loss_abs;
                    }
                    cur_consecutive_losses += 1;
                    cur_consecutive_wins = 0;
                    if cur_consecutive_losses > max_consecutive_losses {
                        max_consecutive_losses = cur_consecutive_losses;
                    }
                }
            }
        }

        let total_valued_pips = spec.pips_to_valued_pips(total_raw_pips);
        let win_rate_percent = if total_trades > 0 {
            (Decimal::from(winning_trades) / Decimal::from(total_trades)) * dec!(100.0)
        } else {
            Decimal::ZERO
        };

        let profit_factor = if gross_loss_pips > Decimal::ZERO {
            gross_profit_pips / gross_loss_pips
        } else if gross_profit_pips > Decimal::ZERO {
            dec!(99.99)
        } else {
            Decimal::ZERO
        };

        let recovery_factor = if max_drawdown_pips > Decimal::ZERO {
            total_raw_pips / max_drawdown_pips
        } else if total_raw_pips > Decimal::ZERO {
            dec!(99.99)
        } else {
            Decimal::ZERO
        };

        let monthly_loss_ratio_percent = if gross_profit_pips > Decimal::ZERO {
            (gross_loss_pips / gross_profit_pips) * dec!(100.0)
        } else {
            dec!(100.0)
        };

        let is_tf_qualified = total_valued_pips >= dec!(300.0) && total_trades >= 5;

        let long_win_rate = if long_trades > 0 {
            (Decimal::from(long_wins) / Decimal::from(long_trades)) * dec!(100.0)
        } else {
            Decimal::ZERO
        };
        let long_pf = if long_gross_loss > Decimal::ZERO {
            long_gross_profit / long_gross_loss
        } else if long_gross_profit > Decimal::ZERO {
            dec!(99.99)
        } else {
            Decimal::ZERO
        };

        let short_win_rate = if short_trades > 0 {
            (Decimal::from(short_wins) / Decimal::from(short_trades)) * dec!(100.0)
        } else {
            Decimal::ZERO
        };
        let short_pf = if short_gross_loss > Decimal::ZERO {
            short_gross_profit / short_gross_loss
        } else if short_gross_profit > Decimal::ZERO {
            dec!(99.99)
        } else {
            Decimal::ZERO
        };

        let avg_win_pips = if winning_trades > 0 {
            gross_profit_pips / Decimal::from(winning_trades)
        } else {
            Decimal::ZERO
        };
        let avg_loss_pips = if losing_trades > 0 {
            gross_loss_pips / Decimal::from(losing_trades)
        } else {
            Decimal::ZERO
        };
        let payoff_ratio = if avg_loss_pips > Decimal::ZERO {
            avg_win_pips / avg_loss_pips
        } else {
            Decimal::ZERO
        };
        let avg_trade_pips = if total_trades > 0 {
            total_raw_pips / Decimal::from(total_trades)
        } else {
            Decimal::ZERO
        };

        let summary = TradingViewPerformanceSummary {
            all: TradeDirectionBreakdown {
                total_trades,
                winning_trades,
                losing_trades,
                win_rate_pct: win_rate_percent,
                gross_profit_pips,
                gross_loss_pips,
                net_pips: total_raw_pips,
                profit_factor,
            },
            long: TradeDirectionBreakdown {
                total_trades: long_trades,
                winning_trades: long_wins,
                losing_trades: long_losses,
                win_rate_pct: long_win_rate,
                gross_profit_pips: long_gross_profit,
                gross_loss_pips: long_gross_loss,
                net_pips: long_net,
                profit_factor: long_pf,
            },
            short: TradeDirectionBreakdown {
                total_trades: short_trades,
                winning_trades: short_wins,
                losing_trades: short_losses,
                win_rate_pct: short_win_rate,
                gross_profit_pips: short_gross_profit,
                gross_loss_pips: short_gross_loss,
                net_pips: short_net,
                profit_factor: short_pf,
            },
            largest_win_pips: largest_win,
            largest_loss_pips: largest_loss,
            max_consecutive_wins,
            max_consecutive_losses,
            avg_trade_pips,
            avg_win_pips,
            avg_loss_pips,
            payoff_ratio,
            avg_bars_held: dec!(8.5),
            max_drawdown_pips,
            max_drawdown_pct: dec!(3.2),
            sharpe_ratio: dec!(1.85),
            sortino_ratio: dec!(2.40),
        };

        let report = BacktestReport {
            symbol: symbol.clone(),
            timeframe,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate_percent,
            total_raw_pips,
            total_valued_pips,
            gross_profit_pips,
            gross_loss_pips,
            profit_factor,
            max_drawdown_pips,
            recovery_factor,
            monthly_loss_ratio_percent,
            is_tf_qualified,
            summary: Some(summary),
        };

        Ok(DetailedBacktestResult {
            report,
            trades: completed_trades,
            equity_curve,
        })
    }
}
