---
description: >-
  Aturan presisi finansial — DILARANG f64 untuk kalkulasi harga, pips, SL/TP, saldo.
  Selalu gunakan rust_decimal::Decimal. Berlaku untuk semua file di workspace ini.
globs: "**/*.rs"
alwaysApply: true
---

# Decimal Precision Rule

## Aturan Mutlak

**DILARANG** menggunakan `f64` atau `f32` untuk:
- Harga pasar (price, bid, ask)
- Pips (SL pips, TP pips, spread pips)
- Stop Loss / Take Profit nilai
- Saldo akun, PnL, margin
- Lot size, volume

**WAJIB** menggunakan `rust_decimal::Decimal`.

## Cara Benar

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ✅ BENAR
let entry_price: Decimal = dec!(1.23456);
let sl_pips: Decimal = dec!(50);
let tp_pips: Decimal = dec!(100);
let rr = tp_pips / sl_pips; // Decimal: presisi penuh

// ❌ SALAH
let entry_price: f64 = 1.23456; // floating point error!
let rr: f64 = 100.0 / 50.0;    // rounding tidak deterministik
```

## Enforcement Otomatis

Rule ini kini diperkuat oleh compiler:
- `clippy::float_arithmetic = "warn"` di `Cargo.toml`
- `disallowed-methods` untuk f64 ops di `clippy.toml`

Lihat detail: [`docs/DATA_INTEGRITY.md`](../../docs/DATA_INTEGRITY.md)
