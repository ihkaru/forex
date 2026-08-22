# 7 Protokol Anti-Pitfall Backtest

> Dimuat on-demand saat bekerja dengan `BacktestService` atau simulasi strategi.
> Setiap simulasi di `crates/core/application/src/services/backtest.rs`
> wajib mematuhi 7 invariant berikut.

---

## TL;DR — Checklist Cepat

```
[ ] 1. Bar-by-bar rolling window — TIDAK ada lookahead bias
[ ] 2. Pending fill hanya saat price touch — BUKAN instant fill
[ ] 3. SL dianggap hit lebih dulu jika SL & TP di bar yang sama
[ ] 4. Spread realistis ≥ 1.2 pips untuk major pair
[ ] 5. 70% in-sample / 30% out-of-sample — TANPA tuning di OOS
[ ] 6. Cross-validate ≥ 6 pairs dari Tier 1 s/d Tier 4
[ ] 7. Exit fixed di TP atau SL — TIDAK ada intervensi
```

---

## Detail 7 Protokol

### 1. Anti Look-Ahead Bias (Rolling Window Bar-by-Bar)

```
Keputusan pada bar i → hanya boleh membaca data bar 0..=(i-1)
Bar i (sedang berjalan) TIDAK boleh dipakai sebagai konfirmasi
```

```rust
// BENAR: hanya baca historical slice
let signal = strategy.analyze(&candles[..=i-1]);

// SALAH: pakai bar current
let signal = strategy.analyze(&candles[..=i]); // ❌ Lookahead!
```

---

### 2. Pending Order Realistic Fill & Expiration

```
BuyLimit/SellLimit → status: Pending
Hanya FILLED jika Ask/Bid menyentuh level limit
Jika 24 jam tidak terisi → status: Expired (PnL = 0)
```

❌ **Instant Fill Bug**: Menganggap pending order aktif seketika setelah dibuat.

---

### 3. Intrabar Worst-Case Resolution

```
Jika dalam 1 bar: HIGH menyentuh TP dan LOW menyentuh SL
→ WAJIB anggap SL hit terlebih dahulu (conservative)
```

Ini mewakili skenario terburuk yang realistis (momentum turun dulu sebelum naik).

---

### 4. Realistic Transaction Costs

```
Spread minimum yang harus disimulasikan:
- Major pairs (EUR/USD, GBP/USD): ≥ 1.2 pips
- Cross pairs: ≥ 2.0 pips
- Gold (XAU/USD): ≥ 3.0 pips

Menggunakan spread realistis broker subscriber (MRG Mega Berjangka)
```

---

### 5. Anti-Overfitting: Walk-Forward Validation

```
In-Sample  (70%): Eksplorasi parameter, optimasi
Out-of-Sample (30%): Evaluasi buta — DILARANG tuning ulang

Parsimony Rule: Lebih sedikit parameter = lebih robust
```

---

### 6. Anti P-Hacking: Multi-Pair Cross Validation

```
DILARANG: Cherry-pick 1 pair yang kebetulan bagus
WAJIB: Test di ≥ 6 pairs, minimal 1 dari setiap Tier (1-4)
```

Strategi hanya valid jika **konsisten profit di semua Tier**, bukan hanya 1 pair.

---

### 7. No-Intervention Guarantee

```
Posisi Running: DILARANG modifikasi SL/TP (trailing stop, BEP parsial)
Exit hanya melalui: TP hit ATAU SL hit
```

Sesuai regulasi TF — intervensi menyebabkan penalty. Lihat `docs/TF_COMPLIANCE.md`.
