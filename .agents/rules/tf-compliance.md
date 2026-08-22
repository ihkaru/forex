---
description: >-
  Aturan kepatuhan Traders Family — berlaku saat membuat, memodifikasi, atau
  memvalidasi sinyal trading. Mencakup pending order, RR ratio, SL limit,
  durasi expiry, dan no-intervention guarantee.
globs: "crates/**/*.rs"
alwaysApply: false
---

# TF Compliance Rule

Setiap sinyal yang dibuat **wajib** lulus `TfComplianceGuard`.

## Quick Reference

| Aturan | Nilai |
|:---|:---|
| Order type | Pending ONLY (BuyLimit/SellLimit/BuyStop/SellStop) |
| Risk:Reward | 1:1.0 ≤ RR ≤ 1:3.0 |
| SL maksimal | SL ≤ 1.5 × TP |
| Sinyal aktif/pair | Maks 2 |
| Durasi expiry | 1–48 jam (Jumat: 96 jam) |
| Intervensi | DILARANG modifikasi SL/TP saat Running |

## Pair Tiers & Constraints

- **Tier 1 (2.0x)**: NZDUSD, AUDUSD, EURGBP, USDCHF — min 10, max 200 pips
- **Tier 2 (1.5x)**: USDCAD, EURUSD, GBPUSD, NZDJPY, CADJPY, AUDJPY — min 15, max 300 pips
- **Tier 3 (1.0x)**: USDJPY, EURJPY, GBPJPY, CHFJPY, EURNZD — min 20, max 400 pips
- **Tier 4 (0.5x)**: XAUUSD — min 30, max 500 pips

Detail lengkap: [`docs/TF_COMPLIANCE.md`](../../docs/TF_COMPLIANCE.md)
