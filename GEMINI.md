# GEMINI.md — Forex Quantitative Workspace Index

> **Ini adalah index, bukan encyclopedia.**
> Setiap section berisi pointer ke dokumentasi detail yang dimuat on-demand.
> Untuk instruksi lengkap suatu topik, baca file yang ditunjuk.

---

## 🎯 Misi

Menghasilkan **≥ 300 Valued Pips + ≥ 5 sinyal settled/bulan** untuk konversi ke Rp
sebagai **Top Analis Traders Family**, sambil memaksimalkan **7-Faktor Scoring** untuk
revenue sharing subscriber Priority Channel.

---

## 🔴 Critical Rules (Selalu Berlaku — Tidak Bisa Diabaikan)

```
[ ] 1. PORT FIRST       : Tulis Trait di domain/src/ports/ SEBELUM adapter konkret
[ ] 2. DECIMAL ONLY     : DILARANG f64 untuk harga/pips/SL/TP → pakai rust_decimal::Decimal
[ ] 3. NO PANIC         : DILARANG panic!(), unwrap(), todo!() di production code → pakai Result
[ ] 4. MAX 600 LINES    : File >600 baris WAJIB dipecah secara modular
[ ] 5. TF COMPLIANCE    : Sinyal WAJIB lulus TfComplianceGuard (Pending Limit, RR 1:1–1:3)
[ ] 6. UTC TIMESTAMPS   : Semua timestamp → chrono::DateTime<Utc>
[ ] 7. ZERO BUILD WARNS : cargo clippy -D warnings, 0 error, 0 warning sebelum selesai
```

> Aturan #2, #3, #7 kini **dieksekusi oleh compiler** via `[workspace.lints]` di `Cargo.toml`
> dan `clippy.toml`. Agen yang melanggar akan diblokir build secara mekanis.

---

## 🚫 Invariant & Larangan Mutlak Analis Traders Family (Hard Constraints)

```
[ ] 1. DILARANG INSTANT EXECUTION : Hanya boleh Pending Order (BuyLimit, SellLimit, BuyStop, SellStop)
[ ] 2. NO-INTERVENTION RULE       : Sinyal RUNNING DILARANG diubah SL/TP atau di-close manual di tengah jalan
[ ] 3. BATAS R:R 1:1.0 s.d. 1:3.0 : DILARANG R:R > 1:3.0. DILARANG SL > 1.5 × TP dalam pips
[ ] 4. MAKSIMAL 2 SINYAL / PAIR   : Kuota maksimal 2 sinyal aktif bersamaan per instrumen
[ ] 5. JARAK PENDING SEARAH       : Tier 1 ≥ 50 pips, Tier 2 ≥ 75 pips, Tier 3/4 ≥ 100 pips (Anti-Martingale)
[ ] 6. SLA SALIN SINYAL (≥ 5 MIN) : Pending order wajib diposting ≥ 5 menit sebelum tersentuh harga pasar
[ ] 7. DURASI KADALUWARSA (EXPIRY): Min 1 jam, Maks 48 jam (Senin–Kamis), Maks 96 jam (Khusus hari Jumat)
[ ] 8. ANTI-HEDGING & INTEGRITAS  : DILARANG multi-account hedging/arbitrage atau membiarkan floating loss ekstrem
```

---

## 📚 Dokumentasi Detail (Baca Sesuai Konteks Task)

| Topik | File | Kapan Dibaca |
|:---|:---|:---|
| Arsitektur hexagonal, DI, komposisi | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Saat tambah layer / adapter baru |
| Regulasi TF, VP, 7 Pilar Scoring | [`docs/TF_COMPLIANCE.md`](docs/TF_COMPLIANCE.md) | Saat membuat / memvalidasi sinyal |
| Integritas data, anti-f64, candle rules | [`docs/DATA_INTEGRITY.md`](docs/DATA_INTEGRITY.md) | Saat bekerja dengan market data |
| 7 Protokol anti-pitfall backtest | [`docs/ANTI_BIAS_BACKTEST.md`](docs/ANTI_BIAS_BACKTEST.md) | Saat modifikasi BacktestService |
| Panduan analis TF lengkap | [`docs/TRADERS_FAMILY_ANALYST_PLAYBOOK.md`](docs/TRADERS_FAMILY_ANALYST_PLAYBOOK.md) | Referensi strategi & scoring |

---

## 🛠️ Perintah Wajib Sebelum Selesai

```bash
cargo fmt --all -- --check          # Format deterministik
cargo clippy --workspace --all-targets -- -D warnings  # 0 warning
cargo test --workspace              # 0 test failure
cargo deny check                    # Dependency audit
```

---

## 🧩 Skills (Runbook On-Demand)

Prosedur multi-langkah tersedia sebagai Skills — dimuat agen otomatis saat relevan:
- **`dukascopy-data-fetch`** — Download & konversi .bi5 → Parquet
- **`tf-signal-publish`** — Pipeline publikasi sinyal ke Trader Family
- **`backtest-runner`** — Jalankan backtest dengan anti-bias checklist
- **`new-strategy-scaffold`** — Scaffold strategi baru (hexagonal-compliant)

---

## ⚡ Stack Ringkas

**Rust Backend**: `tokio` · `sqlx` (TimescaleDB) · `rust_decimal` · `chrono` · `anyhow/thiserror`  
**Data**: Dukascopy `.bi5` → DuckDB+Parquet (historical) · MT5 Bridge + cTrader (live)  
**Frontend**: Svelte 5 Runes · Tailwind CSS v4 · TradingView Lightweight Charts v5  
**CI Gates**: `rustfmt` · `clippy -D warnings` · `cargo-nextest` · `cargo-deny`
