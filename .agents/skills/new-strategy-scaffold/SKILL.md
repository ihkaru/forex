---
name: new-strategy-scaffold
description: >-
  Use this skill when the user asks to create a new trading strategy,
  add a new strategy variant, or scaffold a new strategy crate/module
  following the hexagonal architecture pattern.
---

# New Strategy Scaffold Skill

Scaffold strategi baru yang compliant dengan hexagonal architecture.

## Steps

### 1. Buat Port (Trait) Terlebih Dahulu

```rust
// crates/core/domain/src/ports/strategy.rs (atau extend yang ada)
#[async_trait]
pub trait TradingStrategyPort: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, candles: &[Candle]) -> Option<SignalCandidate>;
    fn validate_setup(&self, candidate: &SignalCandidate) -> Result<(), StrategyError>;
}
```

### 2. Buat Struktur Strategi (Komposisi)

```rust
// crates/core/domain/src/strategies/<strategy_name>/mod.rs

pub struct MyNewStrategy {
    // Komposisi komponen — BUKAN inheritance
    pub swing_detector: SwingPointDetector,
    pub filter: TrendFilter,
    pub config: MyStrategyConfig,
}

impl MyNewStrategy {
    pub fn new(config: MyStrategyConfig) -> Self {
        Self {
            swing_detector: SwingPointDetector::new(config.swing_period),
            filter: TrendFilter::new(config.ema_period),
            config,
        }
    }
}

impl TradingStrategyPort for MyNewStrategy {
    fn name(&self) -> &str { "my-new-strategy" }
    
    fn analyze(&self, candles: &[Candle]) -> Option<SignalCandidate> {
        // Hanya baca data yang sudah closed (anti lookahead)
        let history = candles; // All bars should already be closed
        // ...implementasi...
        None
    }
}
```

### 3. Struktur File (Maks 600 Baris per File)

```
crates/core/domain/src/strategies/my_new_strategy/
├── mod.rs              # pub use, struct definition
├── config.rs           # MyStrategyConfig, default params
├── analyzer.rs         # Core analyze() logic
├── filter.rs           # TrendFilter, entry conditions
└── tests.rs            # Unit tests dengan mock candles
```

### 4. Tulis Unit Test (Mock Data, Zero I/O)

```rust
// crates/core/domain/src/strategies/my_new_strategy/tests.rs
#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    
    fn make_candle(o: &str, h: &str, l: &str, c: &str) -> Candle {
        Candle {
            open: dec!(o.parse()),
            high: dec!(h.parse()),
            low: dec!(l.parse()),
            close: dec!(c.parse()),
            // ...
        }
    }
    
    #[test]
    fn test_no_signal_on_insufficient_data() {
        let strategy = MyNewStrategy::new(MyStrategyConfig::default());
        let candles: Vec<Candle> = vec![]; // Insufficient
        assert!(strategy.analyze(&candles).is_none());
    }
}
```

### 5. Wire ke Composition Root

```rust
// apps/signal-daemon/src/main.rs
let my_strategy = Arc::new(MyNewStrategy::new(config.my_strategy));
let engine = SignalEngineService::new(
    broker,
    publishers,
    storage,
    vec![
        Arc::clone(&my_strategy) as Arc<dyn TradingStrategyPort>,
        // ... strategies lain
    ],
    risk_config,
);
```

### 6. Verifikasi

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p domain -- my_new_strategy --nocapture
```

Lihat contoh: [`examples/pola-n-strategy.rs`](./examples/pola-n-strategy.rs)
