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

## 🧭 Reality & Architecture Reconciliation Ledger (Agentic Context Guard)

> [!IMPORTANT]
> **Wajib Dipahami Seluruh AI Agent**: Catatan ini adalah rekonsiliasi kebenaran arsitektural (*Single Source of Truth*) untuk mencegah agen merujuk pada asumsi dokumen usang:
>
> 1. **Database & Persistensi (Bukan TimescaleDB/DuckDB Eksternal)**:
>    - Mesin persistensi produksi utama adalah **`SqliteStorage` (`crates/adapters/storage-db/src/sqlite.rs`)** dengan SQLite WAL mode (`PRAGMA journal_mode=WAL`), synchronous normal, dan atomic batch transactions.
>    - Dokumentasi lama yang menyebutkan TimescaleDB/DuckDB/Parquet adalah rancangan historis/spesulatif. Seluruh tabel lilin (`candles`), tick (`ticks`), spesifikasi broker (`broker_specs`), sinyal (`signals`), dan order (`orders`) tersimpan di SQLite lokal.
>
> 2. **Pipeline Pasar Terpadu (Bukan In-Memory `candles_map`)**:
>    - Jalur data produksi dari hulu ke hilir menggunakan **`StorageMarketDataAdapter`** (`crates/adapters/storage-db/src/market_adapter.rs`) yang mengikat `StoragePort`, `TickStoragePort`, dan `BrokerSpecPort`.
>    - `AppState.market_adapter` di `apps/api-server/src/state.rs` bertipe `Arc<dyn MarketDataPort>`. Seluruh handler (`audit`, `scorecard`, `monte_carlo`, `eda`, `signals_scan`, `health`) membaca data dari pipeline SQLite persisten ini.
>    - Objek warisan `RealHistoricalMarketAdapter` hanya dipertahankan untuk kompatibilitas script riset offline di `examples/` dan telah **dieliminasi total dari jalur produksi server**.
>
> 3. **Live Market Data Bridge (MetaTrader 4 TCP Socket)**:
>    - Jembatan data broker live menggunakan **`BrokerConnector` (`crates/adapters/broker-connector`)** dengan TCP socket listener di port `5555` (menerima payload JSON `BAR`, `TICK`, dan `SPEC` dari EA MT4).
>    - Memiliki kemampuan **Dual-Persistence**: Menyimpan snapshot ke rolling buffer memori dan secara otomatis menyimpan batch atomic ke `SqliteStorage`.
>
> 4. **Realisme Backtesting & Protokol Anti-Bias**:
>    - **Conservative Dual-Touch Priority**: Jika TP dan SL tersentuh dalam bar yang sama, sistem secara deterministik memprioritaskan SL terlebih dahulu (*worst-case conservative*).
>    - **Same-Bar Intrabar Resolution**: Pending order yang terisi dievaluasi pada bar yang sama menggunakan directional 4-point pathing (Bullish: O→L→H→C, Bearish: O→H→L→C) tanpa time-travel bias.
>    - **Model Friksi Mikrostruktur (Traders Family & MRG Worst-Case Stress Test)**: Dikalibrasi spesifik ke skenario friksi terburuk ekosistem Traders Family / broker B-Book: `simulation_spread_pips: 2.2`, `commission_per_lot_pips: 1.0` ($10/lot), `slippage_pips: 1.0`, swap overnight aktif (`-7.0` points / `-0.7` pips/malam + 3x Rabu), dan Stop Level Enforcement terkonversi akurat via `points_to_pips` (25 points = 2.5 pips).
>    - **Big-O Rolling Chunk Cache**: Query tick historis menggunakan 7-day rolling window cache ($O(\log K)$ partition point search) mereduksi beban disk I/O >95%.
>
> 5. **Isolasi Data Multi-Sumber & Divergensi Lintas-Feed (Cross-Source Determinism)**:
>    - Lilin dan tick dipartisi secara ketat berdasarkan kunci komposit: `(symbol, timeframe, timestamp, source)`. Tidak ada kontaminasi silang (*zero cross-contamination*) antara feed ECN institusional (`DukascopyEcn`) dan feed broker retail (`MrgRealMt4`).
>    - Pengujian instrumen dan periode waktu yang sama lintas sumber data secara deterministik menghasilkan metrik berbeda (misal 133 trades / +469 VP pada Dukascopy vs 119 trades / -38 VP pada MRG) akibat markup spread retail (+2.0 pips) dan execution drag. Paritas antara UI REST API dan Direct Service Layer tetap 100% konsisten untuk masing-masing sumber data.
>
> 6. **Status Rekonsiliasi Dokumen Detail**:
>    - [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): Layer persistensi diselaraskan ke `storage-db/ (SQLite WAL)`.
>    - [`docs/DATA_AUDIT_REAL_VS_SYNTHETIC.md`](docs/DATA_AUDIT_REAL_VS_SYNTHETIC.md): Bagian D diselaraskan ke `SqliteStorage` (WAL mode), TimescaleDB dicatat sebagai skema migrasi opsional masa lalu.
>    - [`docs/E2E_DATA_PIPELINE_AND_TF_AUTOMATION.md`](docs/E2E_DATA_PIPELINE_AND_TF_AUTOMATION.md): Diagram pipeline diselaraskan ke `MarketIngestionService` dan `SqliteStorage` (mengeliminasi rujukan DuckDB/Parquet eksternal).
>    - [`docs/DATA_INTEGRITY.md`](docs/DATA_INTEGRITY.md) & [`docs/ANTI_BIAS_BACKTEST.md`](docs/ANTI_BIAS_BACKTEST.md): 100% selaras dengan implementasi production active.

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
[ ] 8. SRP MODULARITY   : 1 Struct/Modul/Komponen = 1 Tanggung Jawab. Dilarang 'God Files' / monolitik.
[ ] 9. PROD ISOLATION   : DILARANG menjalankan `cargo test --workspace` atau build berat di server ThinkStation (Production). Seluruh testing & validasi WAJIB dilakukan di PC Lokal (Development) agar tidak mengganggu aplikasi production lain.
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
| Pipeline Hulu-Hilir & TF Reverse Engineering | [`docs/E2E_DATA_PIPELINE_AND_TF_AUTOMATION.md`](docs/E2E_DATA_PIPELINE_AND_TF_AUTOMATION.md) | Saat integrasi MT5, Mitmproxy & auto-login TF |

---

## 🔄 Closed-Loop Quantitative Strategy Optimization (Agentic Runbook)

Bagi AI Agent yang melakukan iterasi dan perbaikan strategi kuantitatif, ikuti **5-Tahap Loop Deterministik** di bawah ini secara disiplin:

```
                            AGENTIC OPTIMIZATION CYCLE
                                         │
    ┌────────────────────────────────────┼────────────────────────────────────┐
    ▼                                    ▼                                    ▼
 1. INSPEKSI & FORENSIK         2. MODIFIKASI KODE & KNOBS           3. BENCHMARK MATRIX
 • Jalankan script forensik     • Sesuaikan `formation.rs`           • Jalankan harness 7-pair
 • Query API REST per-pair      • Sesuaikan `strategy.rs`            • Ukur PF, WR, & Valued Pips
 • Cek durasi & peak MFE        • Sesuaikan `detector.rs`            • Bandingkan In-Sample vs OOS
                                         │
                                         ▼
                                4. QUALITY GATE VERIFIKASI
                                • cargo test --workspace (0 fail)
                                • cargo clippy (0 warning)
                                • cargo fmt & AST Fallback scan
```

---

### 📋 1. Cara Melihat & Menginterpretasi Hasil Test (CLI & UI REST API)

Gunakan *tool executables* dan endpoint REST API (yang terhubung langsung dengan Frontend Dashboard UI di port 3000) untuk membaca seluruh metrik performa strategi:

| Tujuan Analisis & Visualisasi | Perintah CLI / REST API Endpoint | Apa yang Harus Diperiksa? |
|:---|:---|:---|
| **Benchmark Cepat 7-Pair (10 Thn)** | `cargo run --example test_institutional_models` | **Profit Factor (target $\ge 2.1$)**, Win Rate ($\ge 45\%$), dan Total Valued Pips (positif). |
| **Audit Bar-by-Bar Losing Trades** | `cargo run --example losing_trades_forensics` | 1. **Peak MFE %**: Jika $>50\%$ lalu kena SL $\implies$ TP terlalu jauh/entry kurang diskon.<br>2. **Held Bars**: Jika $>24$ bar $\implies$ Stagnasi / ranging chop.<br>3. **Immediate SL**: Jika $\le 3$ bar $\implies$ False breakout / counter-trend. |
| **Ringkasan Portofolio (UI Bento Bar)** | `curl -s http://localhost:5000/api/audit/full` | Distribusi total trades, win rate, total VP, dan profit factor antar-tier. |
| **Audit Detail Trade List (UI Table)** | `curl -s http://localhost:5000/api/audit/pair/{symbol}` | Daftar `trades` detail: `exit_reason` (TP/SL), `duration_hours`, `mfe_percent`, dan `valued_pips`. |
| **7-Faktor Scorecard (UI TF Score)** | `cargo run --example eval_7pillars_scorecard`<br>atau `curl -s http://localhost:5000/api/scorecard` | Tabel lengkap 7 pilar: Skor 20/28 (71%), Tier Master/Legend, Recovery Factor, PF, Signal Volume/bln, dan Revenue Share %. |
| **Backtest Matrix Engine (UI Tester)** | `curl -s http://localhost:5000/api/backtest` | Summary metrik backtest multi-pair dan log trade execution. |
| **Visualisasi Chart (TradingView UI)** | `curl -s http://localhost:5000/api/market/candles/{symbol}?timeframe=H1&limit=500` | Integritas bar candlestick OHLCV untuk plotting formasi Pola N di chart. |
| **Monte Carlo Stress Test (UI Risk)** | `curl -s http://localhost:5000/api/monte-carlo/{symbol}` | Distribusi Max Drawdown, Value at Risk (VaR 95%), dan Risk of Ruin (< 1.0%). |
| **Data Quality Scorecard (UI EDA)** | `curl -s http://localhost:5000/api/eda/{symbol}` | Health scorecard data historis: deteksi gap, corrupt bar, dan spread distribution. |

---

### 🛠️ 2. Pemetaan File & Parameter yang Boleh Diubah (Knobs Map)

Saat menyempurnakan strategi, modifikasi file-file berikut sesuai fungsinya:

| File Lokasi | Parameter / Knobs yang Boleh Diubah | Panduan Nilai & Dampak |
|:---|:---|:---|
| [`crates/core/domain/src/models/pola_n/formation.rs`](crates/core/domain/src/models/pola_n/formation.rs) | • `retracement_ratio` (Golden Pocket window)<br>• `suggested_entry` (Level Fibonacci/Retest)<br>• `pip_buffer` (Jarak SL di luar swing)<br>• `min_rr_ratio` (Target R:R 1.0–3.0) | • Entry di $61.8\%$ ($H_1 - 0.618 \times \text{impulse}$) untuk diskon maksimal.<br>• Buffer struktural $2.0$ pips di luar swing.<br>• Target $R:R = 1.49$ s.d. $1.80$. |
| [`crates/core/domain/src/models/pola_n/strategy.rs`](crates/core/domain/src/models/pola_n/strategy.rs) | • `Session Timing Filter` (Jam UTC)<br>• `Candlestick Decisiveness` (Body / Range)<br>• `Triple Trend Alignment` (Fast/Slow/Macro)<br>• `EMA Slope Filter` (Kemiringan tren)<br>• `RSI Window` (Pullback 38–58)<br>• `Volatility Gate` ($\text{ATR}_{14} \ge \text{ATR}_{30}$) | • Jam sesi London/NY: `(7..18).contains(&hour)`.<br>• Body ratio $\ge 0.40$ (tolak Doji).<br>• Fast EMA (12) > Slow (36) > Macro (100).<br>• Slope $> 0$ (Bullish) / $< 0$ (Bearish).<br>• Hanya eksekusi saat volatilitas ekspansi. |
| [`crates/core/domain/src/models/pola_n/detector.rs`](crates/core/domain/src/models/pola_n/detector.rs) | • `left_bars` & `right_bars` (Swing Detector)<br>• Formula kalkulasi indikator teknikal | • Default H1: `(4, 3)` atau `(5, 3)` untuk swing yang terdefinisi bersih. |
| [`crates/core/domain/src/models/risk.rs`](crates/core/domain/src/models/risk.rs) | • `KellyCriterion` (Discrete & Continuous)<br>• `RiskProfile` | • Gunakan Quarter Kelly ($f^* / 4 \le 4.1\%$) untuk menjaga Max Drawdown $< 5\%$. |

---

### 🛡️ 3. Kriteria Kelulusan (Definition of Done)

Sebelum menyatakan tugas selesai, agen **WAJIB** memverifikasi:
1. `cargo test --workspace` $\implies$ **100% PASS (0 Failure)**.
2. `cargo clippy --workspace --all-targets -- -D warnings` $\implies$ **0 Error, 0 Warning**.
3. `cargo fmt --all -- --check` $\implies$ **Deterministic Formatting PASS**.
4. `python3 .agents/scripts/ast_fallback_scanner.py` $\implies$ **0 Violations (PASS)**.
5. Invariant Traders Family (Pending Orders, RR 1.0–3.0, Max 2 Signal/Pair) terbukti 100% patuh.

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

**Rust Backend**: `tokio` · `sqlx` (SQLite WAL mode & atomic transactions) · `rust_decimal` · `chrono` · `anyhow/thiserror`  
**Data**: Dukascopy ECN (historical disk cache seeded to SQLite) · MT4 Bridge (TCP socket port 5555 live dual-persistence) · `StorageMarketDataAdapter` (unified `MarketDataPort`)  
**Frontend**: Svelte 5 Runes · Tailwind CSS v4 · TradingView Lightweight Charts v5 · **Bun v1.4.2** (npm/yarn/pnpm strictly disabled)  
**CI Gates**: `rustfmt` · `clippy -D warnings` · `cargo test --workspace` · `cargo-deny`
