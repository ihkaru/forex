---
name: tf-signal-publish
description: >-
  Use this skill when the user asks to publish, submit, or send a trading signal
  to Traders Family (TF) platform. Also use when troubleshooting TF API errors,
  checking signal status, or validating compliance before submission.
---

# TF Signal Publish Skill

## Pre-Publish Checklist (WAJIB sebelum publish)

```
[ ] 1. Signal type: Pending Order (BuyLimit/SellLimit/BuyStop/SellStop)
[ ] 2. RR ratio: 1:1.0 ≤ RR ≤ 1:3.0
[ ] 3. SL ≤ 1.5 × TP
[ ] 4. Active signals untuk pair ini: < 2
[ ] 5. Pair valid (Tier 1-4): lihat docs/TF_COMPLIANCE.md
[ ] 6. Expiry: 1-48 jam (Jumat: max 96 jam)
```

## Steps

### 1. Buat Signal (via `TfComplianceGuard`)

```rust
let signal = Signal::builder()
    .pair(CurrencyPair::EurUsd)
    .direction(Direction::Buy)
    .order_type(OrderType::BuyLimit)
    .entry(dec!(1.08500))
    .stop_loss(dec!(1.08000))   // 50 pips SL
    .take_profit(dec!(1.09000)) // 50 pips TP (RR 1:1)
    .expiry_hours(24)
    .build()?;

// Validasi otomatis
let guard = TfComplianceGuard::new(TfPairSpec::eurusd());
guard.validate(&signal)?; // Error jika melanggar invariant
```

### 2. Publish via `TraderFamilyPublisher`

```bash
# Test koneksi TF API
cargo run -p apps -- signal publish --dry-run

# Publish sinyal nyata
cargo run -p apps -- signal publish --signal-id <UUID>
```

### 3. Verifikasi Status

```bash
# Cek status sinyal di database lokal
cargo run -p apps -- signal status --pair EURUSD --limit 5

# Cek log publikasi
grep "published" logs/signal-daemon.log | tail -20
```

## Error TF API Umum

| Error | Penyebab | Solusi |
|:---|:---|:---|
| `RR_RATIO_EXCEEDED` | RR > 1:3 | Sesuaikan TP lebih dekat |
| `INSTANT_EXECUTION` | Sinyal bukan pending | Ubah ke BuyLimit/SellLimit |
| `MAX_SIGNALS_REACHED` | ≥ 2 sinyal aktif di pair | Tunggu sinyal sebelumnya settle |
| `SL_EXCEEDS_LIMIT` | SL > max tier | Kurangi SL sesuai tier pair |

## Setelah Publish

- Catat signal_id di database (`SqlxStorage`)
- Monitor via Telegram notifier
- JANGAN modifikasi SL/TP setelah sinyal Running

Referensi lengkap: [`docs/TF_COMPLIANCE.md`](../../docs/TF_COMPLIANCE.md)
