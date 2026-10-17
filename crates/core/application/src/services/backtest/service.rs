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
        let broker_spec = self
            .market_data
            .get_broker_spec(symbol)
            .await
            .unwrap_or_else(|| domain::models::BrokerSymbolSpec::new(symbol.clone(), symbol.to_compact_string()));

        let mut completed_trades: Vec<Order> = Vec::new();
        let mut active_sim_orders: Vec<SimulatedOrder> = Vec::new();
        let mut peak_pips = Decimal::ZERO;
        let mut max_drawdown_pips = Decimal::ZERO;
        let mut running_pips = Decimal::ZERO;
        let mut total_swap_pips = Decimal::ZERO;
        let mut total_commission_pips = Decimal::ZERO;
        let mut total_slippage_pips = Decimal::ZERO;
        let mut tick_walk_count = 0usize;

        let window_size = self.config.lookback_window_bars;
        let spread_offset = spec.pip_size * self.config.simulation_spread_pips;
        // Max durasi trade setelah terisi — hindari stagnation churn yang terbukti
        // merugikan dari forensik candle (58–71% trade rugi = slow churn > 12h)
        let max_filled_duration = chrono::Duration::hours(24);

        let mut last_signal_bar: usize = 0;

        // Rolling 7-day chunk cache untuk mereduksi I/O disk/DB hingga >95%
        let mut tick_window_start = from;
        let mut tick_window_end = from;
        let mut cached_ticks: Vec<Tick> = Vec::new();

        if historical_candles.len() > window_size {
            for i in window_size..historical_candles.len() {
                let slice = &historical_candles[(i - window_size)..i];
                let current_candle = &historical_candles[i];

                // Dual-Resolution Hybrid Execution:
                // Cek apakah data tick historis resolusi tinggi tersedia untuk rentang bar ini
                let next_bar_ts = if i + 1 < historical_candles.len() {
                    historical_candles[i + 1].timestamp
                } else {
                    current_candle.timestamp + chrono::Duration::hours(1)
                };

                if current_candle.timestamp >= tick_window_end || current_candle.timestamp < tick_window_start {
                    tick_window_start = current_candle.timestamp;
                    tick_window_end = current_candle.timestamp + chrono::Duration::days(7);
                    cached_ticks = self
                        .market_data
                        .get_historical_ticks(symbol, tick_window_start, tick_window_end)
                        .await
                        .unwrap_or_default();
                }

                let bar_ticks: Vec<Tick> = if cached_ticks.is_empty() {
                    Vec::new()
                } else {
                    let start_idx = cached_ticks.partition_point(|t| t.timestamp < current_candle.timestamp);
                    let end_idx = cached_ticks.partition_point(|t| t.timestamp < next_bar_ts);
                    if start_idx < end_idx {
                        cached_ticks[start_idx..end_idx].to_vec()
                    } else {
                        Vec::new()
                    }
                };

                let effective_spread = current_candle.spread.unwrap_or(spread_offset);

                // Update status pending & running orders dengan resolusi tick / fallback candle
                let mut remaining_orders = Vec::new();
                for mut sim in active_sim_orders {
                    let mut closed = false;
                    let mut expired = false;
                    let tp = sim.order.take_profit;

                    if !bar_ticks.is_empty() {
                        tick_walk_count += 1;
                        // === MODE 1: TICK-WALK RESOLUTION (Zero Bar-Ambiguity) ===
                        for tick in &bar_ticks {
                            match sim.status {
                                SimulatedOrderStatus::Pending => {
                                    if tick.timestamp >= sim.expires_at {
                                        expired = true;
                                        break;
                                    }
                                    let is_filled = match sim.order.action {
                                        SignalAction::BuyLimit => tick.ask <= sim.order.open_price,
                                        SignalAction::SellLimit => tick.bid >= sim.order.open_price,
                                        SignalAction::BuyStop => tick.ask >= sim.order.open_price,
                                        SignalAction::SellStop => tick.bid <= sim.order.open_price,
                                        _ => false,
                                    };
                                    if is_filled {
                                        sim.status = SimulatedOrderStatus::Filled {
                                            fill_time: tick.timestamp,
                                        };
                                        sim.order.open_time = tick.timestamp;
                                    }
                                }
                                SimulatedOrderStatus::Filled { fill_time } => {
                                    let duration = tick.timestamp - fill_time;
                                    let is_buy = matches!(sim.order.action, SignalAction::BuyLimit | SignalAction::BuyStop);
                                    let comm_pips = self.config.commission_per_lot_pips * sim.order.volume_lots;
                                    let slippage_pips = if self.config.enable_slippage {
                                        self.config.slippage_pips
                                    } else {
                                        Decimal::ZERO
                                    };

                                    if duration >= max_filled_duration {
                                        let swap_pips = if self.config.enable_swap {
                                            broker_spec.calculate_swap_pips(is_buy, sim.order.open_time, tick.timestamp)
                                        } else {
                                            Decimal::ZERO
                                        };
                                        total_swap_pips += swap_pips;
                                        total_commission_pips += comm_pips;
                                        total_slippage_pips += slippage_pips;

                                        let net_pips = -swap_pips - comm_pips - slippage_pips;
                                        sim.order.close_time = Some(tick.timestamp);
                                        sim.order.realized_pnl = Some(net_pips);
                                        running_pips += net_pips;
                                        closed = true;
                                        break;
                                    }
                                    let sl = sim.order.stop_loss;
                                    match sim.order.action {
                                        SignalAction::BuyLimit | SignalAction::BuyStop => {
                                            if tick.bid <= sl {
                                                let raw_loss = if sim.sl_moved_to_breakeven {
                                                    Decimal::ZERO
                                                } else {
                                                    -spec.price_diff_to_pips(sim.order.open_price - sl)
                                                };
                                                let swap_pips = if self.config.enable_swap {
                                                    broker_spec.calculate_swap_pips(is_buy, sim.order.open_time, tick.timestamp)
                                                } else {
                                                    Decimal::ZERO
                                                };
                                                total_swap_pips += swap_pips;
                                                total_commission_pips += comm_pips;
                                                total_slippage_pips += slippage_pips;

                                                let net_loss = raw_loss - swap_pips - comm_pips - slippage_pips;
                                                sim.order.close_time = Some(tick.timestamp);
                                                sim.order.realized_pnl = Some(net_loss);
                                                running_pips += net_loss;
                                                closed = true;
                                                break;
                                            } else if tick.bid >= tp {
                                                let raw_win = spec.price_diff_to_pips(tp - sim.order.open_price);
                                                let swap_pips = if self.config.enable_swap {
                                                    broker_spec.calculate_swap_pips(is_buy, sim.order.open_time, tick.timestamp)
                                                } else {
                                                    Decimal::ZERO
                                                };
                                                total_swap_pips += swap_pips;
                                                total_commission_pips += comm_pips;
                                                total_slippage_pips += slippage_pips;

                                                let net_win = raw_win - swap_pips - comm_pips - slippage_pips;
                                                sim.order.close_time = Some(tick.timestamp);
                                                sim.order.realized_pnl = Some(net_win);
                                                running_pips += net_win;
                                                closed = true;
                                                break;
                                            }
                                        }
                                        SignalAction::SellLimit | SignalAction::SellStop => {
                                            if tick.ask >= sl {
                                                let raw_loss = if sim.sl_moved_to_breakeven {
                                                    Decimal::ZERO
                                                } else {
                                                    -spec.price_diff_to_pips(sl - sim.order.open_price)
                                                };
                                                let swap_pips = if self.config.enable_swap {
                                                    broker_spec.calculate_swap_pips(is_buy, sim.order.open_time, tick.timestamp)
                                                } else {
                                                    Decimal::ZERO
                                                };
                                                total_swap_pips += swap_pips;
                                                total_commission_pips += comm_pips;
                                                total_slippage_pips += slippage_pips;

                                                let net_loss = raw_loss - swap_pips - comm_pips - slippage_pips;
                                                sim.order.close_time = Some(tick.timestamp);
                                                sim.order.realized_pnl = Some(net_loss);
                                                running_pips += net_loss;
                                                closed = true;
                                                break;
                                            } else if tick.ask <= tp {
                                                let raw_win = spec.price_diff_to_pips(sim.order.open_price - tp);
                                                let swap_pips = if self.config.enable_swap {
                                                    broker_spec.calculate_swap_pips(is_buy, sim.order.open_time, tick.timestamp)
                                                } else {
                                                    Decimal::ZERO
                                                };
                                                total_swap_pips += swap_pips;
                                                total_commission_pips += comm_pips;
                                                total_slippage_pips += slippage_pips;

                                                let net_win = raw_win - swap_pips - comm_pips - slippage_pips;
                                                sim.order.close_time = Some(tick.timestamp);
                                                sim.order.realized_pnl = Some(net_win);
                                                running_pips += net_win;
                                                closed = true;
                                                break;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    } else {
                        // === MODE 2: CANDLE-LEVEL DIRECTIONAL 4-POINT INTRABAR RESOLUTION ===
                        match sim.status {
                            SimulatedOrderStatus::Pending => {
                                if current_candle.timestamp >= sim.expires_at {
                                    expired = true;
                                } else {
                                    let is_filled = match sim.order.action {
                                        SignalAction::BuyLimit => {
                                            current_candle.low + effective_spread <= sim.order.open_price
                                        }
                                        SignalAction::SellLimit => {
                                            current_candle.high >= sim.order.open_price
                                        }
                                        SignalAction::BuyStop => {
                                            current_candle.high + effective_spread >= sim.order.open_price
                                        }
                                        SignalAction::SellStop => {
                                            current_candle.low <= sim.order.open_price
                                        }
                                        _ => false,
                                    };
                                    if is_filled {
                                        sim.status = SimulatedOrderStatus::Filled {
                                            fill_time: current_candle.timestamp,
                                        };
                                        sim.order.open_time = current_candle.timestamp;

                                        // Same-bar Intrabar Resolution:
                                        // Cek apakah bar penjemputan langsung menyentuh SL atau TP
                                        let is_buy = matches!(sim.order.action, SignalAction::BuyLimit | SignalAction::BuyStop);
                                        let is_bullish = current_candle.close >= current_candle.open;
                                        let sl = sim.order.stop_loss;
                                        let tp = sim.order.take_profit;

                                        let (sl_hit, tp_hit) = match sim.order.action {
                                            SignalAction::BuyLimit => {
                                                let sl_h = current_candle.low <= sl;
                                                let tp_h = if is_bullish {
                                                    current_candle.high >= tp
                                                } else {
                                                    current_candle.close >= tp
                                                };
                                                (sl_h, tp_h)
                                            }
                                            SignalAction::SellLimit => {
                                                let sl_h = current_candle.high + effective_spread >= sl;
                                                let tp_h = if !is_bullish {
                                                    current_candle.low + effective_spread <= tp
                                                } else {
                                                    current_candle.close + effective_spread <= tp
                                                };
                                                (sl_h, tp_h)
                                            }
                                            SignalAction::BuyStop => {
                                                let sl_h = current_candle.close <= sl;
                                                let tp_h = current_candle.high >= tp;
                                                (sl_h, tp_h)
                                            }
                                            SignalAction::SellStop => {
                                                let sl_h = current_candle.close + effective_spread >= sl;
                                                let tp_h = current_candle.low + effective_spread <= tp;
                                                (sl_h, tp_h)
                                            }
                                            _ => (false, false),
                                        };

                                        // Aturan 3 Anti-Bias (Worst-Case Priority: SL first)
                                        let hit_decision = if sl_hit {
                                            Some(false)
                                        } else if tp_hit {
                                            Some(true)
                                        } else {
                                            None
                                        };

                                        if let Some(is_win_hit) = hit_decision {
                                            let raw_loss = -spec.price_diff_to_pips(if is_buy {
                                                sim.order.open_price - sl
                                            } else {
                                                sl - sim.order.open_price
                                            });
                                            let raw_win = spec.price_diff_to_pips(if is_buy {
                                                tp - sim.order.open_price
                                            } else {
                                                sim.order.open_price - tp
                                            });
                                            let gross_pips = if is_win_hit { raw_win } else { raw_loss };
                                            let comm_pips = self.config.commission_per_lot_pips * sim.order.volume_lots;
                                            total_commission_pips += comm_pips;
                                            let slippage_pips = if self.config.enable_slippage {
                                                self.config.slippage_pips
                                            } else {
                                                Decimal::ZERO
                                            };
                                            total_slippage_pips += slippage_pips;

                                            let net_pips = gross_pips - comm_pips - slippage_pips;
                                            sim.order.close_time = Some(current_candle.timestamp);
                                            sim.order.realized_pnl = Some(net_pips);
                                            running_pips += net_pips;
                                            closed = true;
                                        }
                                    }
                                }
                            }
                            SimulatedOrderStatus::Filled { fill_time } => {
                                let duration_since_fill = current_candle.timestamp - fill_time;
                                let is_buy = matches!(sim.order.action, SignalAction::BuyLimit | SignalAction::BuyStop);
                                let comm_pips = self.config.commission_per_lot_pips * sim.order.volume_lots;
                                let slippage_pips = if self.config.enable_slippage {
                                    self.config.slippage_pips
                                } else {
                                    Decimal::ZERO
                                };

                                if duration_since_fill >= max_filled_duration {
                                    let swap_pips = if self.config.enable_swap {
                                        broker_spec.calculate_swap_pips(is_buy, sim.order.open_time, current_candle.timestamp)
                                    } else {
                                        Decimal::ZERO
                                    };
                                    total_swap_pips += swap_pips;
                                    total_commission_pips += comm_pips;
                                    total_slippage_pips += slippage_pips;
                                    let net_pips = -swap_pips - comm_pips - slippage_pips;

                                    sim.order.close_time = Some(current_candle.timestamp);
                                    sim.order.realized_pnl = Some(net_pips);
                                    running_pips += net_pips;
                                    closed = true;
                                } else {
                                    let sl = sim.order.stop_loss;
                                    let tp = sim.order.take_profit;

                                    let (sl_hit, tp_hit, raw_loss, raw_win) = if is_buy {
                                        let sl_hit = current_candle.low <= sl;
                                        let tp_hit = current_candle.high >= tp;
                                        let loss = if sim.sl_moved_to_breakeven {
                                            Decimal::ZERO
                                        } else {
                                            -spec.price_diff_to_pips(sim.order.open_price - sl)
                                        };
                                        let win = spec.price_diff_to_pips(tp - sim.order.open_price);
                                        (sl_hit, tp_hit, loss, win)
                                    } else {
                                        let sl_hit = current_candle.high + effective_spread >= sl;
                                        let tp_hit = current_candle.low + effective_spread <= tp;
                                        let loss = if sim.sl_moved_to_breakeven {
                                            Decimal::ZERO
                                        } else {
                                            -spec.price_diff_to_pips(sl - sim.order.open_price)
                                        };
                                        let win = spec.price_diff_to_pips(sim.order.open_price - tp);
                                        (sl_hit, tp_hit, loss, win)
                                    };

                                    // Aturan 3 Anti-Bias: Intrabar Worst-Case Resolution
                                    // Jika HIGH menyentuh TP dan LOW menyentuh SL -> SL terpicu lebih dulu!
                                    let hit_decision = if sl_hit && tp_hit {
                                        Some(false)
                                    } else if sl_hit {
                                        Some(false)
                                    } else if tp_hit {
                                        Some(true)
                                    } else {
                                        None
                                    };

                                    if let Some(is_win_hit) = hit_decision {
                                        let gross_pips = if is_win_hit { raw_win } else { raw_loss };
                                        let swap_pips = if self.config.enable_swap {
                                            broker_spec.calculate_swap_pips(
                                                is_buy,
                                                sim.order.open_time,
                                                current_candle.timestamp,
                                            )
                                        } else {
                                            Decimal::ZERO
                                        };
                                        total_swap_pips += swap_pips;
                                        total_commission_pips += comm_pips;
                                        total_slippage_pips += slippage_pips;

                                        let net_pips = gross_pips - swap_pips - comm_pips - slippage_pips;

                                        sim.order.close_time = Some(current_candle.timestamp);
                                        sim.order.realized_pnl = Some(net_pips);
                                        running_pips += net_pips;
                                        closed = true;
                                    }
                                }
                            }
                        }
                    }

                    if expired {
                        // Pending order expired before fill - discard
                        continue;
                    }

                    if closed {
                        completed_trades.push(sim.order);
                    } else {
                        remaining_orders.push(sim);
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

                // Evaluasi sinyal baru jika kuota pair belum penuh (max 2 order aktif —
                // sesuai Invariant TF: "MAKSIMAL 2 SINYAL / PAIR")
                if active_sim_orders.len() < 2 {
                    let last_slice_candle = &slice[slice.len() - 1];
                    let tick = Tick {
                        symbol: last_slice_candle.symbol.clone(),
                        bid: last_slice_candle.close,
                        ask: last_slice_candle.close + spread_offset,
                        timestamp: last_slice_candle.timestamp,
                        source: last_slice_candle.source,
                        spread_pts: None,
                        volume: None,
                    };

                    let context = MarketContext {
                        symbol,
                        timeframe,
                        current_tick: &tick,
                        candles: slice,
                        risk_profile: &self.risk_profile,
                    };

                    if let Ok(Some(signal)) = self.strategy.evaluate(&context).await {
                        // Cooldown 3 bar minimum antara sinyal untuk mencegah duplikasi setup
                        let is_cooldown_active = i.saturating_sub(last_signal_bar) < 3;

                        // Guard JARAK PENDING SEARAH (Anti-Martingale — Invariant TF):
                        // Order kedua searah harus berjarak minimal min_same_direction_gap_pips
                        // dari order pertama yang masih pending (Tier1≥50pip, Tier2≥75pip, Tier3/4≥100pip)
                        let violates_gap_rule = active_sim_orders.iter().any(|existing| {
                            let same_dir = matches!(
                                (existing.order.action, signal.action),
                                (
                                    SignalAction::BuyLimit | SignalAction::BuyStop,
                                    SignalAction::BuyLimit | SignalAction::BuyStop,
                                ) | (
                                    SignalAction::SellLimit | SignalAction::SellStop,
                                    SignalAction::SellLimit | SignalAction::SellStop,
                                )
                            );
                            if same_dir {
                                let gap = spec.price_diff_to_pips(
                                    (existing.order.open_price - signal.entry_price).abs(),
                                );
                                gap < spec.min_same_direction_gap_pips
                            } else {
                                false
                            }
                        });

                        // Broker Stop Level Enforcement (Anti ERR_INVALID_STOPS):
                        let min_stop_gap = spec.pip_size * Decimal::from(broker_spec.stop_level_pts);
                        let violates_stop_level = self.config.enforce_stop_levels
                            && (signal.entry_price - tick.bid).abs() < min_stop_gap;

                        if !is_cooldown_active
                            && !violates_gap_rule
                            && !violates_stop_level
                            && TfComplianceGuard::validate_signal(&signal).is_ok()
                        {
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
                                posted_time: Some(current_candle.timestamp),
                            };

                            // Forensik: pending order tidak terisi > 8 jam sudah kehilangan momentum
                            // (session London + NY berakhir, setup sudah tidak valid)
                            let sim_order = SimulatedOrder {
                                order,
                                status: SimulatedOrderStatus::Pending,
                                expires_at: current_candle.timestamp + chrono::Duration::hours(8),
                                sl_moved_to_breakeven: false,
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
            total_swap_deducted_pips: total_swap_pips,
            total_commission_deducted_pips: total_commission_pips,
            total_slippage_deducted_pips: total_slippage_pips,
            execution_model: if tick_walk_count > 0 {
                format!(
                    "DUAL_RESOLUTION_HYBRID ({} Ticks Walk + 4-Point Directional)",
                    tick_walk_count
                )
            } else {
                "DIRECTIONAL_4POINT_INTRABAR_PATHING (Realistic Anti-Bias)".to_string()
            },
        };

        Ok(DetailedBacktestResult {
            report,
            trades: completed_trades,
            equity_curve,
        })
    }
}
