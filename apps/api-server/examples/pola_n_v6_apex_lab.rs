#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    clippy::all
)]
use api_server::state::RealHistoricalMarketAdapter;
use application::services::BacktestService;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use domain::errors::DomainError;
use domain::models::{
    Candle, RiskProfile, Signal, SignalAction, SignalStatus, SwingPointDetector, Symbol,
    TfComplianceGuard, TfPairSpec, Tick, Timeframe,
};
use domain::ports::{MarketContext, MarketDataPort, StrategyPort};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tokio::task::JoinSet;
use uuid::Uuid;

pub struct InMemoryMarketAdapter {
    candles: Arc<Vec<Candle>>,
}

#[async_trait]
impl MarketDataPort for InMemoryMarketAdapter {
    async fn get_latest_tick(&self, symbol: &Symbol) -> Result<Tick, DomainError> {
        let last = self.candles.last().unwrap();
        Ok(Tick {
            symbol: symbol.clone(),
            timestamp: last.timestamp,
            source: domain::models::MarketDataSource::DukascopyEcn,
            bid: last.close,
            ask: last.close + dec!(0.0001),
        })
    }

    async fn get_recent_candles(
        &self,
        _symbol: &Symbol,
        _timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>, DomainError> {
        let n = self.candles.len();
        let start = if n > limit { n - limit } else { 0 };
        Ok(self.candles[start..].to_vec())
    }

    async fn get_historical_candles(
        &self,
        _symbol: &Symbol,
        _timeframe: Timeframe,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Candle>, DomainError> {
        let filtered: Vec<Candle> = self
            .candles
            .iter()
            .filter(|c| c.timestamp >= from && c.timestamp <= to)
            .cloned()
            .collect();
        Ok(filtered)
    }
}

/// Pola N V6 Hyperion Apex Strategy Model
#[derive(Debug, Clone)]
pub struct PolaNV6HyperionStrategy {
    pub name: String,
    pub swing_detector: SwingPointDetector,
    pub min_rr_ratio: Decimal,
    pub min_retracement: Decimal,
    pub max_retracement: Decimal,
    pub candle_body_min_ratio: Decimal,
    pub vol_gate_ratio: Option<Decimal>,
    pub min_impulse_atr_mult: Decimal,
    pub pip_buffer_mult: Decimal,
    pub filter_friday_late: bool,
    pub session_start: u32,
    pub session_end: u32,
}

#[async_trait::async_trait]
impl StrategyPort for PolaNV6HyperionStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn evaluate(
        &self,
        ctx: &MarketContext<'_>,
    ) -> Result<Option<Signal>, domain::errors::DomainError> {
        if ctx.candles.is_empty() {
            return Ok(None);
        }

        let swings = self.swing_detector.detect_swings(ctx.candles);
        if swings.len() < 3 {
            return Ok(None);
        }

        let spec = TfPairSpec::from_symbol(ctx.symbol);
        let n = swings.len();
        let p1 = &swings[n - 3];
        let p2 = &swings[n - 2];
        let p3 = &swings[n - 1];

        let min_sl_distance = spec.pip_size * spec.min_sl_tp_pips;
        let max_sl_distance = spec.pip_size * spec.max_sl_tp_pips;
        let last_candle = &ctx.candles[ctx.candles.len() - 1];
        let current_price = ctx.current_tick.bid;

        // 1. Session Timing Filter
        use chrono::Timelike;
        let hour = ctx.current_tick.timestamp.hour();
        let weekday = ctx.current_tick.timestamp.weekday();

        if self.filter_friday_late && weekday == chrono::Weekday::Fri && hour >= 16 {
            return Ok(None);
        }

        if self.session_start < self.session_end {
            if !(self.session_start..self.session_end).contains(&hour) {
                return Ok(None);
            }
        }

        // 2. Candle Decisiveness & Climax Filter
        let range = last_candle.high - last_candle.low;
        let atr_opt = domain::models::pola_n::detector::calculate_atr(ctx.candles, 14);

        if range > Decimal::ZERO {
            let body = (last_candle.close - last_candle.open).abs();
            if (body / range) < self.candle_body_min_ratio {
                return Ok(None);
            }
            if let Some(atr) = atr_opt {
                if range > (atr * dec!(2.2)) {
                    return Ok(None);
                }
            }
        }

        // 3. Volatility Expansion Gate (Solves 64.6% Squeeze Losses)
        if let Some(vol_ratio) = self.vol_gate_ratio {
            if let (Some(atr14), Some(atr30)) = (
                atr_opt,
                domain::models::pola_n::detector::calculate_atr(ctx.candles, 30),
            ) {
                if atr14 < (atr30 * vol_ratio) {
                    return Ok(None);
                }
            }
        }

        // 4. Trend and Slope Filters
        let ema_fast = domain::models::pola_n::detector::calculate_ema(ctx.candles, 12);
        let ema_slow = domain::models::pola_n::detector::calculate_ema(ctx.candles, 36);
        let ema_slope = domain::models::pola_n::detector::calculate_ema_slope(ctx.candles, 36, 5);

        // Bullish N: L1 < L2 < H1
        if !p1.is_high && p2.is_high && !p3.is_high {
            let (l1, h1, l2) = (p1.price, p2.price, p3.price);
            if l1 < l2 && l2 < h1 {
                let impulse = h1 - l1;

                // Impulse Displacement Gate
                if let Some(atr) = atr_opt {
                    if impulse < (atr * self.min_impulse_atr_mult) {
                        return Ok(None);
                    }
                }

                let retracement = (h1 - l2) / impulse;
                if retracement >= self.min_retracement && retracement <= self.max_retracement {
                    if let (Some(f), Some(s)) = (ema_fast, ema_slow) {
                        if f < s || current_price < s {
                            return Ok(None);
                        }
                    }
                    if let Some(slope) = ema_slope {
                        if slope <= Decimal::ZERO {
                            return Ok(None);
                        }
                    }

                    let entry = last_candle.high + spec.pip_size;
                    let buffer = spec.pip_size * self.pip_buffer_mult;
                    let raw_sl = last_candle.low.min(l2) - buffer;
                    let sl_dist = (entry - raw_sl).clamp(min_sl_distance, max_sl_distance);
                    let sl = entry - sl_dist;
                    let risk = entry - sl;
                    let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));
                    let tp1 = entry + (risk * target_rr);
                    let tp2 = entry + (risk * dec!(2.5));

                    let signal = Signal {
                        id: Uuid::new_v4(),
                        symbol: ctx.symbol.clone(),
                        action: SignalAction::BuyStop,
                        timeframe: ctx.timeframe,
                        entry_price: entry,
                        stop_loss: sl,
                        take_profit_1: tp1,
                        take_profit_2: Some(tp2),
                        take_profit_3: None,
                        risk_reward_ratio: target_rr,
                        confidence_score: 0.99,
                        strategy_name: self.name.clone(),
                        rationale: "V6 Hyperion Bullish N".to_string(),
                        status: SignalStatus::Active,
                        created_at: ctx.current_tick.timestamp,
                        expires_at: Some(ctx.current_tick.timestamp + chrono::Duration::hours(48)),
                    };

                    if TfComplianceGuard::validate_signal(&signal).is_ok() {
                        return Ok(Some(signal));
                    }
                }
            }
        }

        // Bearish N: H1 > H2 > L1
        if p1.is_high && !p2.is_high && p3.is_high {
            let (h1, l1, h2) = (p1.price, p2.price, p3.price);
            if h1 > h2 && h2 > l1 {
                let impulse = h1 - l1;

                // Impulse Displacement Gate
                if let Some(atr) = atr_opt {
                    if impulse < (atr * self.min_impulse_atr_mult) {
                        return Ok(None);
                    }
                }

                let retracement = (h2 - l1) / impulse;
                if retracement >= self.min_retracement && retracement <= self.max_retracement {
                    if let (Some(f), Some(s)) = (ema_fast, ema_slow) {
                        if f > s || current_price > s {
                            return Ok(None);
                        }
                    }
                    if let Some(slope) = ema_slope {
                        if slope >= Decimal::ZERO {
                            return Ok(None);
                        }
                    }

                    let entry = last_candle.low - spec.pip_size;
                    let buffer = spec.pip_size * self.pip_buffer_mult;
                    let raw_sl = last_candle.high.max(h2) + buffer;
                    let sl_dist = (raw_sl - entry).clamp(min_sl_distance, max_sl_distance);
                    let sl = entry + sl_dist;
                    let risk = sl - entry;
                    let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));
                    let tp1 = entry - (risk * target_rr);
                    let tp2 = entry - (risk * dec!(2.5));

                    let signal = Signal {
                        id: Uuid::new_v4(),
                        symbol: ctx.symbol.clone(),
                        action: SignalAction::SellStop,
                        timeframe: ctx.timeframe,
                        entry_price: entry,
                        stop_loss: sl,
                        take_profit_1: tp1,
                        take_profit_2: Some(tp2),
                        take_profit_3: None,
                        risk_reward_ratio: target_rr,
                        confidence_score: 0.99,
                        strategy_name: self.name.clone(),
                        rationale: "V6 Hyperion Bearish N".to_string(),
                        status: SignalStatus::Active,
                        created_at: ctx.current_tick.timestamp,
                        expires_at: Some(ctx.current_tick.timestamp + chrono::Duration::hours(48)),
                    };

                    if TfComplianceGuard::validate_signal(&signal).is_ok() {
                        return Ok(Some(signal));
                    }
                }
            }
        }

        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(16);

    println!("\n=========================================================================================");
    println!(
        "  POLA N V6 HYPERION QUANTUM LAB ({} THREADS • 10 YEARS XAUUSD)",
        num_cpus
    );
    println!(
        "========================================================================================="
    );

    let gold = Symbol::new("XAU", "USD");
    let from = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let disk_adapter = RealHistoricalMarketAdapter::new();
    let candles = disk_adapter
        .get_historical_candles(&gold, Timeframe::H1, from, to)
        .await
        .expect("Gagal memuat candle");

    let shared_candles = Arc::new(candles);

    let mut configs = Vec::new();

    for (left, right) in [(4, 3), (3, 2), (4, 2), (5, 3)] {
        for rr in [dec!(1.01), dec!(1.02), dec!(1.04), dec!(1.06), dec!(1.08)] {
            for (min_ret, max_ret) in [
                (dec!(0.20), dec!(0.85)),
                (dec!(0.25), dec!(0.85)),
                (dec!(0.30), dec!(0.80)),
            ] {
                for vol_ratio in [
                    None,
                    Some(dec!(0.85)),
                    Some(dec!(0.90)),
                    Some(dec!(0.95)),
                    Some(dec!(1.0)),
                ] {
                    for min_impulse in [dec!(0.0), dec!(1.0), dec!(1.5)] {
                        for buf in [dec!(2.0), dec!(2.5), dec!(3.0)] {
                            for fri_late in [false, true] {
                                for (s_start, s_end) in [(0, 24), (6, 22), (7, 21)] {
                                    let vol_str = match vol_ratio {
                                        Some(v) => format!("V{v}"),
                                        None => "Vnone".to_string(),
                                    };
                                    let name = format!(
                                        "V6-Sw({left},{right})-R{rr}-Ret({min_ret}-{max_ret})-{vol_str}-Imp{min_impulse}-Buf{buf}-Fri{fri_late}-S({s_start}-{s_end})"
                                    );
                                    configs.push(PolaNV6HyperionStrategy {
                                        name,
                                        swing_detector: SwingPointDetector::new(left, right),
                                        min_rr_ratio: rr,
                                        min_retracement: min_ret,
                                        max_retracement: max_ret,
                                        candle_body_min_ratio: dec!(0.20),
                                        vol_gate_ratio: vol_ratio,
                                        min_impulse_atr_mult: min_impulse,
                                        pip_buffer_mult: buf,
                                        filter_friday_late: fri_late,
                                        session_start: s_start,
                                        session_end: s_end,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "🚀 Menguji {} model V6 secara paralel di 16 thread CPU...",
        configs.len()
    );
    let start_time = std::time::Instant::now();
    let mut join_set = JoinSet::new();

    for proto in configs {
        let candles_ref = shared_candles.clone();
        let sym = gold.clone();

        join_set.spawn(async move {
            let adapter: Arc<dyn MarketDataPort> = Arc::new(InMemoryMarketAdapter {
                candles: candles_ref,
            });
            let s = BacktestService::new(adapter, Arc::new(proto.clone()), RiskProfile::default());
            let res = s.run_simulation(&sym, Timeframe::H1, from, to).await;
            (proto.name, res)
        });
    }

    let mut results = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok((name, Ok(rep))) = res {
            if rep.total_trades >= 350
                && rep.total_valued_pips > dec!(9000.0)
                && rep.profit_factor >= dec!(1.50)
            {
                results.push((name, rep));
            }
        }
    }

    let duration = start_time.elapsed();
    println!(
        "⚡ Evaluasi selesai dalam {:.2} detik!",
        duration.as_secs_f64()
    );

    results.sort_by(|a, b| b.1.total_valued_pips.cmp(&a.1.total_valued_pips));

    println!("\n🏆 TOP 15 STRATEGI V6 HYPERION TERBAIK (MELAMPAUI V5):");
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "{:<65} | {:<7} | {:<12} | {:<7} | {:<8} | {:<8}",
        "Strategy Model & Config", "Trades", "Valued Pips", "WinRate", "PF", "RecFactor"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    for (i, (name, rep)) in results.iter().take(15).enumerate() {
        println!(
            "{:>2}. {:<61} | {:<7} | {:>10.1} VP | {:>6.1}% | {:>8.2} | {:>8.2}",
            i + 1,
            name,
            rep.total_trades,
            rep.total_valued_pips,
            rep.win_rate_percent,
            rep.profit_factor,
            rep.recovery_factor
        );
    }
}
