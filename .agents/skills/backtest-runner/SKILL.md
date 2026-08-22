---
name: backtest-runner
description: >-
  Use this skill when the user asks to run a backtest, simulate a strategy,
  or validate strategy performance on historical data. Includes anti-bias
  checklist verification and result interpretation.
---

# Backtest Runner Skill

## Pre-Run Checklist (WAJIB)

Sebelum jalankan backtest apapun, verifikasi 7 protokol anti-bias:

```
[ ] 1. BAR-BY-BAR    : Tidak ada akses ke data bar yang belum tutup
[ ] 2. REALISTIC FILL: Pending fill hanya saat price touch level
[ ] 3. WORST-CASE    : SL dianggap hit dulu jika SL & TP di bar yang sama
[ ] 4. SPREAD COST   : Spread ≥ 1.2 pips major, ≥ 2.0 cross, ≥ 3.0 Gold
[ ] 5. 70/30 SPLIT   : Pisahkan dataset, JANGAN tune di OOS
[ ] 6. MULTI-PAIR    : Test minimal 6 pairs dari Tier 1-4
[ ] 7. NO INTERVENSI : Tidak ada trailing stop / BEP parsial
```

Detail: [`references/anti-bias-checklist.md`](./references/anti-bias-checklist.md)

## Steps

### 1. Pastikan Data Tersedia

```bash
# Cek ketersediaan data Parquet
ls data/historical/EURUSD/
ls data/historical/GBPUSD/
# ... dst untuk semua 6+ pairs
```

### 2. Jalankan Backtest

```bash
# Backtest single pair (development)
cargo run -p apps -- backtest \
  --strategy pola-n \
  --pair EURUSD \
  --from 2018-01-01 \
  --to 2023-12-31 \
  --spread 1.5

# Backtest multi-pair (validasi final)
cargo run -p apps -- backtest \
  --strategy pola-n \
  --pairs EURUSD,GBPUSD,NZDUSD,USDJPY,XAUUSD,EURJPY \
  --from 2018-01-01 \
  --to 2023-12-31 \
  --out-of-sample-from 2022-01-01
```

### 3. Interpretasi Hasil

Metrik yang harus diperiksa:

| Metrik | Target Minimum | Ideal (Skor TF 4 Poin) |
|:---|:---:|:---:|
| Recovery Factor | ≥ 5.0 | ≥ 8.0 |
| Profit Factor | ≥ 1.5 | ≥ 2.1 |
| Win Rate | ≥ 45% | ≥ 55% |
| Max Drawdown | < 25% | < 15% |
| Avg Monthly VP | ≥ 200 VP | ≥ 300 VP |

### 4. Validasi Cross-Pair Consistency

Strategi TIDAK VALID jika:
- Profit di 1-2 pair saja (cherry-pick)
- Recovery Factor < 5.0 di salah satu Tier
- OOS performance jauh di bawah IS performance (overfitting)

## Cara Baca Warning Backtest

```
WARN: "SL and TP touched same bar — conservative SL assumed"
→ Normal — simulator sudah benar menerapkan worst-case
```

Referensi: [`docs/ANTI_BIAS_BACKTEST.md`](../../docs/ANTI_BIAS_BACKTEST.md)
