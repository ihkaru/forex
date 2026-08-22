# Integritas & Kualitas Data Pasar

> Protokol wajib untuk semua data market yang masuk ke sistem.
> Dimuat on-demand saat bekerja dengan data feed, scraper, atau adapter data.

---

## 1. Sumber Data Resmi

| Mode | Sumber | Adapter |
|:---|:---|:---|
| **Backtest (Historical Lab)** | Dukascopy Bank SA — True Tick sejak 2003 | `crates/scrapers/dukascopy-rs` |
| **Live / Forward Test** | MT5 EA Bridge + cTrader Open API | `crates/adapters/broker-connector` |

> ⚠️ **PANTANGAN MUTLAK**: DILARANG backtest jangka panjang dengan data riwayat MT5 ritel
> (shallow history + pseudo-tick interpolation → curve-fitting bias).

---

## 2. Normalisasi Timezone

Semua timestamp **WAJIB UTC**:

```rust
// BENAR
let ts: chrono::DateTime<chrono::Utc> = raw_ts.with_timezone(&chrono::Utc);

// SALAH — timezone lokal masuk ke domain
let ts: chrono::NaiveDateTime = raw_ts; // ❌ Tidak ada timezone info
```

---

## 3. Presisi Finansial — Anti-f64 Rule

**DILARANG** `f64` untuk harga, pips, lot, SL, TP, saldo, PnL.  
**WAJIB** `rust_decimal::Decimal`.

```rust
// BENAR
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let price: Decimal = dec!(1.2345);
let sl_pips: Decimal = dec!(50.0);
let rr_ratio: Decimal = sl_pips / tp_pips; // Decimal arithmetic

// SALAH — f64 precision loss
let price: f64 = 1.2345; // ❌ Floating point rounding error!
```

> 💡 Enforcement: `clippy::float_arithmetic = "warn"` di `Cargo.toml` akan catch
> accidental f64 arithmetic. Lihat `clippy.toml` untuk `disallowed-methods`.

---

## 4. Spread Anomaly Filter

`DataIntegrityValidator` wajib memvalidasi setiap tick:

| Kondisi | Action |
|:---|:---|
| `Ask < Bid` | Tolak — data korup |
| `Spread > 5.0 pips` | Tolak — rollover spike / anomali |
| `Volume < 0` | Tolak — invalid |

---

## 5. Candle Mathematical Invariants

Setiap `Candle` yang dibuat atau diterima harus memenuhi:

```
High ≥ Low
High ≥ Open
High ≥ Close
Low  ≤ Open
Low  ≤ Close
Volume ≥ 0
```

Implementasi: `crates/core/domain/src/entities/candle.rs` — method `validate()`.

---

## 6. Protokol Anti-Silent Fallback (Fail-Fast & Parse, Don't Validate)

> 🚨 **Prinsip Utama: Crash / Error 100x Lebih Baik daripada Silent Data Corruption.**
> Mengganti data hilang/korup dengan nilai default sembarangan dapat menyebabkan eksekusi trading di harga yang salah!

### ❌ DILARANG (Anti-Patterns):
1. **Hardcoded Price Fallback**: `.unwrap_or(dec!(1.0850))` saat data candle kosong.
2. **Silent Zero Default**: `.unwrap_or_default()` pada entitas finansial (harga/pips/volume).
3. **Silent Symbol Substitution**: `Symbol::from_symbol_str(&s).unwrap_or_else(|| Symbol::new("EUR", "USD"))`.
4. **Error Swallowing**: `.parse::<f64>().unwrap_or(0.0)`.

### ✅ WAJIB (Correct Patterns):
1. **Propagate Errors dengan `?`**:
   ```rust
   let symbol = Symbol::from_symbol_str(&symbol_str)
       .ok_or_else(|| DomainError::AdapterError(format!("Symbol '{}' invalid", symbol_str)))?;
   ```
2. **Gunakan `filter_map` untuk Optional Collections**:
   ```rust
   let realized_pnl: Vec<Decimal> = trades.iter().filter_map(|t| t.realized_pnl).collect();
   ```
3. **DILARANG `#[derive(Default)]` pada Tipe Domain Finansial** (`Price`, `Signal`, `Order`, `Candle`).

### 🛠️ Mekanisme Enforcement Deterministik:
- **`clippy.toml`**: `disallowed-methods` untuk `Option::unwrap_or_default` & `Result::unwrap_or_default`.
- **`.agents/scripts/ast_fallback_scanner.py`**: Scanner AST yang berjalan otomatis pada setiap Stop Hook dan CI.
- **GitHub Actions CI**: Memblokir build jika ada pelanggaran fallback.
