---
description: >-
  Aturan mutlak integritas data — DILARANG silent fallback (unwrap_or_default, unwrap_or(0),
  hardcoded prices) pada data finansial. Selalu propagate error dengan Result/'?' atau filter_map.
globs: "**/*.rs"
alwaysApply: true
---

# Anti-Silent Fallback Rule (Data Integrity #1)

## Aturan Mutlak

**Crash / Error 100x LEBIH BAIK daripada Silent Data Corruption.**

Di domain finansial/trading, mengganti data yang hilang atau korup dengan default dummy (seperti `0.0`, `1.0850`, atau empty string) **DILARANG KERAS** karena dapat menyebabkan eksekusi order pada harga yang salah atau sinyal palsu ke subscriber.

## Pola yang DILARANG vs WAJIB

### 1. Data Symbol / Pair
```rust
// ❌ DILARANG: Fallback diam-diam ke EURUSD jika parsing gagal
let sym = Symbol::from_symbol_str(&s).unwrap_or_else(|| Symbol::new("EUR", "USD"));

// ✅ WAJIB: Propagate error secara eksplisit
let sym = Symbol::from_symbol_str(&s)
    .ok_or_else(|| DomainError::AdapterError(format!("Symbol tidak valid: {}", s)))?;
```

### 2. Harga & Nilai Finansial
```rust
// ❌ DILARANG: Hardcoded harga jika candle kosong
let last_close = candles.last().map(|c| c.close).unwrap_or(dec!(1.0850));

// ✅ WAJIB: Return error jika data tidak mencukupi
let last_close = candles.last().map(|c| c.close)
    .ok_or_else(|| DomainError::MarketDataError("Data candle kosong".into()))?;
```

### 3. Parsing Float / Numeric
```rust
// ❌ DILARANG: Silent 0.0 jika parsing gagal
let val = str_val.parse::<f64>().unwrap_or(0.0);

// ✅ WAJIB: Gunakan Decimal dan propagate parse error
let val = Decimal::from_str(&str_val)?;
```

### 4. Optional Trade PnL
```rust
// ❌ DILARANG: Anggap trade belum close sebagai 0 PnL
let pnls: Vec<Decimal> = trades.iter().map(|t| t.realized_pnl.unwrap_or(Decimal::ZERO)).collect();

// ✅ WAJIB: Gunakan filter_map untuk hanya mengambil trade yang settled
let pnls: Vec<Decimal> = trades.iter().filter_map(|t| t.realized_pnl).collect();
```

## Enforcement Deterministik

Rule ini ditegakkan oleh:
1. `clippy.toml` (`disallowed-methods`)
2. `.agents/scripts/ast_fallback_scanner.py` (dipanggil oleh Antigravity Stop Hook)
3. GitHub Actions CI pipeline
