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

/// Advanced Institutional V5 Strategy Model Exploring Community & SMC Confluences
#[derive(Debug, Clone)]
pub struct PolaNV5InstitutionalStrategy {
    pub name: String,
    pub swing_detector: SwingPointDetector,
    pub min_rr_ratio: Decimal,
    pub min_retracement: Decimal,
    pub max_retracement: Decimal,
    pub candle_body_min_ratio: Decimal,
    pub enable_slope_filter: bool,
    pub enable_macro_100_ema: bool,
    pub enable_rsi_filter: bool,
    pub session_start: u32,
    pub session_end: u32,
    pub max_tp_pips: Option<Decimal>,
    pub enable_liquidity_wick_check: bool,
}

#[async_trait::async_trait]
impl StrategyPort for PolaNV5InstitutionalStrategy {
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
        if self.session_start < self.session_end {
            if !(self.session_start..self.session_end).contains(&hour) {
                return Ok(None);
            }
        }

        // 2. Candle Decisiveness & Climax Filter
        let range = last_candle.high - last_candle.low;
        if range > Decimal::ZERO {
            let body = (last_candle.close - last_candle.open).abs();
            if (body / range) < self.candle_body_min_ratio {
                return Ok(None);
            }
            if let Some(atr) = domain::models::pola_n::detector::calculate_atr(ctx.candles, 14) {
                if range > (atr * dec!(2.2)) {
                    return Ok(None);
                }
            }
        }

        // 3. Technical Indicators
        let ema_fast = domain::models::pola_n::detector::calculate_ema(ctx.candles, 12);
        let ema_slow = domain::models::pola_n::detector::calculate_ema(ctx.candles, 36);
        let ema_macro = domain::models::pola_n::detector::calculate_ema(ctx.candles, 100);
        let ema_slope = domain::models::pola_n::detector::calculate_ema_slope(ctx.candles, 36, 5);
        let rsi = domain::models::pola_n::detector::calculate_rsi(ctx.candles, 14);

        // Bullish N: L1 < L2 < H1
        if !p1.is_high && p2.is_high && !p3.is_high {
            let (l1, h1, l2) = (p1.price, p2.price, p3.price);
            if l1 < l2 && l2 < h1 {
                let impulse = h1 - l1;
                let retracement = (h1 - l2) / impulse;
                if retracement >= self.min_retracement && retracement <= self.max_retracement {
                    // EMA Trend Alignment
                    if let (Some(f), Some(s)) = (ema_fast, ema_slow) {
                        if f < s || current_price < s {
                            return Ok(None);
                        }
                    }
                    if self.enable_macro_100_ema {
                        if let Some(m) = ema_macro {
                            if current_price < m {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_slope_filter {
                        if let Some(slope) = ema_slope {
                            if slope <= Decimal::ZERO {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_rsi_filter {
                        if let Some(r) = rsi {
                            // RSI Pullback zone: avoid overbought (>68)
                            if r > dec!(68.0) || r < dec!(35.0) {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_liquidity_wick_check {
                        // Point 3 candle must show buying rejection wick
                        let p3_candle = &ctx.candles[ctx.candles.len() - 1];
                        let p3_range = p3_candle.high - p3_candle.low;
                        if p3_range > Decimal::ZERO {
                            let lower_wick = p3_candle.open.min(p3_candle.close) - p3_candle.low;
                            if (lower_wick / p3_range) < dec!(0.15) {
                                return Ok(None);
                            }
                        }
                    }

                    let entry = last_candle.high + spec.pip_size;
                    let raw_sl = last_candle.low.min(l2) - (spec.pip_size * dec!(2.0));
                    let sl_dist = (entry - raw_sl).clamp(min_sl_distance, max_sl_distance);
                    let sl = entry - sl_dist;
                    let risk = entry - sl;
                    let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));

                    let mut tp1 = entry + (risk * target_rr);
                    if let Some(max_tp) = self.max_tp_pips {
                        let max_tp_dist = spec.pip_size * max_tp;
                        if (tp1 - entry) > max_tp_dist {
                            tp1 = entry + max_tp_dist;
                        }
                    }
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
                        confidence_score: 0.98,
                        strategy_name: self.name.clone(),
                        rationale: "V5 Institutional Bullish N".to_string(),
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
                let retracement = (h2 - l1) / impulse;
                if retracement >= self.min_retracement && retracement <= self.max_retracement {
                    // EMA Trend Alignment
                    if let (Some(f), Some(s)) = (ema_fast, ema_slow) {
                        if f > s || current_price > s {
                            return Ok(None);
                        }
                    }
                    if self.enable_macro_100_ema {
                        if let Some(m) = ema_macro {
                            if current_price > m {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_slope_filter {
                        if let Some(slope) = ema_slope {
                            if slope >= Decimal::ZERO {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_rsi_filter {
                        if let Some(r) = rsi {
                            // RSI Pullback zone: avoid oversold (<32)
                            if r < dec!(32.0) || r > dec!(65.0) {
                                return Ok(None);
                            }
                        }
                    }
                    if self.enable_liquidity_wick_check {
                        // Point 3 candle must show selling rejection wick
                        let p3_candle = &ctx.candles[ctx.candles.len() - 1];
                        let p3_range = p3_candle.high - p3_candle.low;
                        if p3_range > Decimal::ZERO {
                            let upper_wick = p3_candle.high - p3_candle.open.max(p3_candle.close);
                            if (upper_wick / p3_range) < dec!(0.15) {
                                return Ok(None);
                            }
                        }
                    }

                    let entry = last_candle.low - spec.pip_size;
                    let raw_sl = last_candle.high.max(h2) + (spec.pip_size * dec!(2.0));
                    let sl_dist = (raw_sl - entry).clamp(min_sl_distance, max_sl_distance);
                    let sl = entry + sl_dist;
                    let risk = sl - entry;
                    let target_rr = self.min_rr_ratio.clamp(dec!(1.0), dec!(3.0));

                    let mut tp1 = entry - (risk * target_rr);
                    if let Some(max_tp) = self.max_tp_pips {
                        let max_tp_dist = spec.pip_size * max_tp;
                        if (entry - tp1) > max_tp_dist {
                            tp1 = entry - max_tp_dist;
                        }
                    }
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
                        confidence_score: 0.98,
                        strategy_name: self.name.clone(),
                        rationale: "V5 Institutional Bearish N".to_string(),
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
        "  POLA N V5 ADVANCED INSTITUTIONAL QUANTUM LAB ({} THREADS • 10 YEARS)",
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
        for rr in [dec!(1.02), dec!(1.05), dec!(1.08), dec!(1.10), dec!(1.15)] {
            for (min_ret, max_ret) in [
                (dec!(0.25), dec!(0.85)),
                (dec!(0.30), dec!(0.85)),
                (dec!(0.38), dec!(0.786)),
            ] {
                for macro_100 in [false, true] {
                    for rsi_gate in [false, true] {
                        for (s_start, s_end) in [(0, 24), (7, 21), (8, 18)] {
                            for wick_check in [false, true] {
                                for max_tp in [None, Some(dec!(100.0)), Some(dec!(120.0))] {
                                    let name = format!(
                                        "V5-Sw({left},{right})-R{rr}-Ret({min_ret}-{max_ret})-M100({macro_100})-RSI({rsi_gate})-Ses({s_start}-{s_end})-Wick({wick_check})"
                                    );
                                    configs.push(PolaNV5InstitutionalStrategy {
                                        name,
                                        swing_detector: SwingPointDetector::new(left, right),
                                        min_rr_ratio: rr,
                                        min_retracement: min_ret,
                                        max_retracement: max_ret,
                                        candle_body_min_ratio: dec!(0.20),
                                        enable_slope_filter: true,
                                        enable_macro_100_ema: macro_100,
                                        enable_rsi_filter: rsi_gate,
                                        session_start: s_start,
                                        session_end: s_end,
                                        max_tp_pips: max_tp,
                                        enable_liquidity_wick_check: wick_check,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("🚀 Menguji {} model V5 secara paralel...", configs.len());
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
            if rep.total_trades >= 300
                && rep.total_valued_pips > dec!(7500.0)
                && rep.profit_factor >= dec!(1.40)
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

    println!("\n🏆 TOP 15 STRATEGI V5 INSTITUTIONAL TERBAIK (MELAMPAUI V4):");
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
