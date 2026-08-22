---
description: >-
  Protokol anti-bias untuk backtest — berlaku saat memodifikasi BacktestService,
  simulasi strategi, atau menulis test backtest. Mencakup 7 invariant anti-pitfall.
globs: "**/backtest*,**/simulation*"
alwaysApply: false
---

# Anti-Bias Backtest Rule

Setiap perubahan pada `BacktestService` atau simulasi WAJIB mematuhi 7 invariant:

```
[ ] 1. BAR-BY-BAR    : bar i hanya baca data ≤ i-1 (zero lookahead)
[ ] 2. REALISTIC FILL: pending fill hanya saat price touch, bukan instant
[ ] 3. WORST-CASE    : SL hit dulu jika SL & TP di bar yang sama
[ ] 4. SPREAD COST   : spread ≥ 1.2 pips major pair, ≥ 2.0 cross, ≥ 3.0 Gold
[ ] 5. 70/30 SPLIT   : 70% in-sample, 30% OOS tanpa tuning ulang
[ ] 6. MULTI-PAIR    : validate ≥ 6 pairs dari semua Tier (1–4)
[ ] 7. NO INTERVENSI : exit hanya via TP atau SL hit
```

Detail: [`docs/ANTI_BIAS_BACKTEST.md`](../../docs/ANTI_BIAS_BACKTEST.md)
