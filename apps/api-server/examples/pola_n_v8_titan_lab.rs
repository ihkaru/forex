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
use chrono::{DateTime, TimeZone, Utc};
use domain::errors::DomainError;
use domain::models::{
    Candle, RiskProfile, Signal, SignalAction, SignalStatus, SwingPointDetector, Symbol,
    TfComplianceGuard, TfPairSpec, Tick, Timeframe,
};
use domain::ports::{MarketContext, MarketDataPort, StrategyPort};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;
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

pub fn calculate_chop(candles: &[Candle], period: usize) -> Option<Decimal> {
    if candles.len() < period + 1 {
        return None;
    }
    let slice = &candles[candles.len() - period..];

    let mut sum_atr1 = Decimal::ZERO;
    let mut max_high = slice[0].high;
    let mut min_low = slice[0].low;

    for c in slice {
        if c.high > max_high {
            max_high = c.high;
        }
        if c.low < min_low {
            min_low = c.low;
        }
        sum_atr1 += c.high - c.low;
    }

    let high_low_diff = max_high - min_low;
    if high_low_diff <= Decimal::ZERO || sum_atr1 <= Decimal::ZERO {
        return None;
    }

    let sum_atr_f64 = sum_atr1.to_f64()?;
    let hl_diff_f64 = high_low_diff.to_f64()?;
    let period_f64 = period as f64;

    let ratio = sum_atr_f64 / hl_diff_f64;
    if ratio <= 0.0 {
        return None;
    }

    let chop_val = 100.0 * (ratio.log10() / period_f64.log10());
    Decimal::from_f64_retain(chop_val)
}

pub fn calculate_adx(candles: &[Candle], period: usize) -> Option<Decimal> {
    if candles.len() < (period * 2) {
        return None;
    }
    let n = candles.len();
    let mut tr_sum = Decimal::ZERO;
    let mut dm_plus_sum = Decimal::ZERO;
    let mut dm_minus_sum = Decimal::ZERO;

    for i in (n - period)..n {
        let curr = &candles[i];
        let prev = &candles[i - 1];

        let tr = (curr.high - curr.low)
            .max((curr.high - prev.close).abs())
            .max((curr.low - prev.close).abs());
        tr_sum += tr;

        let up_move = curr.high - prev.high;
        let down_move = prev.low - curr.low;

        if up_move > down_move && up_move > Decimal::ZERO {
            dm_plus_sum += up_move;
        }
        if down_move > up_move && down_move > Decimal::ZERO {
            dm_minus_sum += down_move;
        }
    }

    if tr_sum <= Decimal::ZERO {
        return None;
    }

    let di_plus = (dm_plus_sum / tr_sum) * dec!(100.0);
    let di_minus = (dm_minus_sum / tr_sum) * dec!(100.0);
    let di_diff = (di_plus - di_minus).abs();
    let di_sum = di_plus + di_minus;

    if di_sum <= Decimal::ZERO {
        return None;
    }

    Some((di_diff / di_sum) * dec!(100.0))
}

#[derive(Debug, Clone)]
pub struct PolaNV8TitanStrategy {
    pub name: String,
    pub swing_detector: SwingPointDetector,
    pub min_rr_ratio: Decimal,
    pub min_retracement: Decimal,
    pub max_retracement: Decimal,
    pub max_chop_index: Option<Decimal>,
    pub min_adx: Option<Decimal>,
    pub min_body_ratio: Decimal,
    pub min_displacement_atr: Decimal,
    pub pip_buffer_mult: Decimal,
    pub entry_discount_ratio: Option<Decimal>, // None = Breakout Stop, Some(0.50) = Discount Limit
    pub enable_macro_ema: bool,
}

#[async_trait::async_trait]
impl StrategyPort for PolaNV8TitanStrategy {
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

        // Candle Decisiveness & Climax Filter
        let range = last_candle.high - last_candle.low;
        let atr_opt = domain::models::pola_n::detector::calculate_atr(ctx.candles, 14);

        if range > Decimal::ZERO {
            let body = (last_candle.close - last_candle.open).abs();
            if (body / range) < self.min_body_ratio {
                return Ok(None);
            }
            if let Some(atr) = atr_opt {
                if range > (atr * dec!(2.2)) {
                    return Ok(None);
                }
            }
        }

        // Anti-Chop Gates
        if let Some(max_chop) = self.max_chop_index {
            if let Some(chop) = calculate_chop(ctx.candles, 14) {
                if chop > max_chop {
                    return Ok(None);
                }
            }
        }

        if let Some(min_adx) = self.min_adx {
            if let Some(adx) = calculate_adx(ctx.candles, 14) {
                if adx < min_adx {
                    return Ok(None);
                }
            }
        }

        // Trend and Slope
        let ema_fast = domain::models::pola_n::detector::calculate_ema(ctx.candles, 12);
        let ema_slow = domain::models::pola_n::detector::calculate_ema(ctx.candles, 36);
        let ema_macro = domain::models::pola_n::detector::calculate_ema(ctx.candles, 100);
        let ema_slope = domain::models::pola_n::detector::calculate_ema_slope(ctx.candles, 36, 5);

        // Bullish N: L1 < L2 < H1
        if !p1.is_high && p2.is_high && !p3.is_high {
            let (l1, h1, l2) = (p1.price, p2.price, p3.price);
            if l1 < l2 && l2 < h1 {
                let impulse = h1 - l1;

                if let Some(atr) = atr_opt {
                    if impulse < (atr * self.min_displacement_atr) {
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
                    if self.enable_macro_ema {
                        if let Some(m) = ema_macro {
                            if current_price < m {
                                return Ok(None);
                            }
                        }
                    }
                    if let Some(slope) = ema_slope {
                        if slope <= Decimal::ZERO {
                            return Ok(None);
                        }
                    }

                    // Entry calculation (Breakout vs Discount Retest)
                    let entry = match self.entry_discount_ratio {
                        Some(disc) => l2 + (impulse * disc),      // Discount Limit Entry
                        None => last_candle.high + spec.pip_size, // Breakout Stop
                    };

                    let buffer = spec.pip_size * self.pip_buffer_mult;
                    let raw_sl = last_candle.low.min(l2) - buffer;
                    let sl_dist = (entry - raw_sl).clamp(min_sl_distance, max_sl_distance);
                    let sl = entry - sl_dist;
                    let risk = entry - sl;
                    let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));
                    let tp1 = entry + (risk * target_rr);
                    let tp2 = entry + (risk * dec!(2.5));

                    let action = if entry >= current_price {
                        SignalAction::BuyStop
                    } else {
                        SignalAction::BuyLimit
                    };

                    let signal = Signal {
                        id: Uuid::new_v4(),
                        symbol: ctx.symbol.clone(),
                        action,
                        timeframe: ctx.timeframe,
                        entry_price: entry,
                        stop_loss: sl,
                        take_profit_1: tp1,
                        take_profit_2: Some(tp2),
                        take_profit_3: None,
                        risk_reward_ratio: target_rr,
                        confidence_score: 0.99,
                        strategy_name: self.name.clone(),
                        rationale: "V8 Titan Bullish N".to_string(),
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

                if let Some(atr) = atr_opt {
                    if impulse < (atr * self.min_displacement_atr) {
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
                    if self.enable_macro_ema {
                        if let Some(m) = ema_macro {
                            if current_price > m {
                                return Ok(None);
                            }
                        }
                    }
                    if let Some(slope) = ema_slope {
                        if slope >= Decimal::ZERO {
                            return Ok(None);
                        }
                    }

                    let entry = match self.entry_discount_ratio {
                        Some(disc) => h2 - (impulse * disc),     // Premium Limit Entry
                        None => last_candle.low - spec.pip_size, // Breakdown Stop
                    };

                    let buffer = spec.pip_size * self.pip_buffer_mult;
                    let raw_sl = last_candle.high.max(h2) + buffer;
                    let sl_dist = (raw_sl - entry).clamp(min_sl_distance, max_sl_distance);
                    let sl = entry + sl_dist;
                    let risk = sl - entry;
                    let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));
                    let tp1 = entry - (risk * target_rr);
                    let tp2 = entry - (risk * dec!(2.5));

                    let action = if entry <= current_price {
                        SignalAction::SellStop
                    } else {
                        SignalAction::SellLimit
                    };

                    let signal = Signal {
                        id: Uuid::new_v4(),
                        symbol: ctx.symbol.clone(),
                        action,
                        timeframe: ctx.timeframe,
                        entry_price: entry,
                        stop_loss: sl,
                        take_profit_1: tp1,
                        take_profit_2: Some(tp2),
                        take_profit_3: None,
                        risk_reward_ratio: target_rr,
                        confidence_score: 0.99,
                        strategy_name: self.name.clone(),
                        rationale: "V8 Titan Bearish N".to_string(),
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
        "  POLA N V8 TITAN QUANTUM LEAP LAB ({} THREADS • 10 YEARS XAUUSD)",
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
        for rr in [
            dec!(1.01),
            dec!(1.02),
            dec!(1.04),
            dec!(1.08),
            dec!(1.15),
            dec!(1.25),
        ] {
            for (min_ret, max_ret) in [(dec!(0.15), dec!(0.85)), (dec!(0.20), dec!(0.85))] {
                for chop_max in [None, Some(dec!(60.0)), Some(dec!(58.0))] {
                    for adx_min in [None, Some(dec!(18.0)), Some(dec!(20.0))] {
                        for body in [dec!(0.15), dec!(0.20), dec!(0.25)] {
                            for disp in [dec!(0.8), dec!(1.0), dec!(1.2)] {
                                for buf in [dec!(2.0), dec!(2.5), dec!(3.0)] {
                                    for macro_ema in [false, true] {
                                        let name = format!(
                                            "V8-Sw({left},{right})-R{rr}-Ret({min_ret}-{max_ret})-Chop{:?}-Adx{:?}-B{body}-D{disp}-Buf{buf}-M{macro_ema}",
                                            chop_max, adx_min
                                        );
                                        configs.push(PolaNV8TitanStrategy {
                                            name,
                                            swing_detector: SwingPointDetector::new(left, right),
                                            min_rr_ratio: rr,
                                            min_retracement: min_ret,
                                            max_retracement: max_ret,
                                            max_chop_index: chop_max,
                                            min_adx: adx_min,
                                            min_body_ratio: body,
                                            min_displacement_atr: disp,
                                            pip_buffer_mult: buf,
                                            entry_discount_ratio: None,
                                            enable_macro_ema: macro_ema,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "🚀 Menguji {} model V8 Titan secara paralel di 16 thread CPU...",
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
            let res = s
                .run_simulation_detailed(&sym, Timeframe::H1, from, to)
                .await;
            (proto.name, res)
        });
    }

    let mut results = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok((name, Ok(detail))) = res {
            let mut monthly_vp: BTreeMap<String, Decimal> = BTreeMap::new();
            for t in &detail.trades {
                let pnl = t.realized_pnl.unwrap_or(Decimal::ZERO);
                let m = t.open_time.format("%Y-%m").to_string();
                *monthly_vp.entry(m).or_default() += pnl * dec!(0.50);
            }

            let mut neg_months = 0;
            let mut pos_months = 0;
            for (_m, vp) in &monthly_vp {
                if *vp < Decimal::ZERO {
                    neg_months += 1;
                } else if *vp > Decimal::ZERO {
                    pos_months += 1;
                }
            }

            if detail.report.total_trades >= 400
                && detail.report.total_valued_pips > dec!(10000.0)
                && detail.report.profit_factor >= dec!(1.55)
            {
                results.push((name, detail.report, pos_months, neg_months));
            }
        }
    }

    let duration = start_time.elapsed();
    println!(
        "⚡ Evaluasi selesai dalam {:.2} detik!\n",
        duration.as_secs_f64()
    );

    // Sort by Total Valued Pips Descending
    results.sort_by(|a, b| b.1.total_valued_pips.cmp(&a.1.total_valued_pips));

    println!("🏆 TOP 20 STRATEGI V8 TITAN DENGAN VALUED PIPS & PROFIT FACTOR TERTINGGI:");
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────────────");
    println!(
        "{:<65} | {:<7} | {:<12} | {:<7} | {:<5} | {:<11} | {:<8}",
        "Strategy Model & Config",
        "Trades",
        "Valued Pips",
        "WinRate",
        "PF",
        "Pos/Neg Bulan",
        "RecFactor"
    );
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────────────");

    for (i, (name, rep, pos, neg)) in results.iter().take(20).enumerate() {
        let pos_neg_str = format!("{pos} Pos / {neg} Neg");
        println!(
            "{:>2}. {:<61} | {:<7} | {:>10.1} VP | {:>6.1}% | {:>5.2} | {:<11} | {:>8.2}",
            i + 1,
            name,
            rep.total_trades,
            rep.total_valued_pips,
            rep.win_rate_percent,
            rep.profit_factor,
            pos_neg_str,
            rep.recovery_factor
        );
    }
}
