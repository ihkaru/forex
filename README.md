# 📈 Forex Quantitative Research Lab & Autonomous Signal Pipeline (2026 Edition)

Selamat datang di repository **Forex Quantitative Research Lab, Multi-Year Backtesting Engine, Scraper Pipeline, dan Autonomous Signal Daemon** berbasis **Rust & Svelte 5**.

Repository ini dirancang secara khusus untuk mencapai tujuan finansial nyata: **menghasilkan pendapatan puluhan hingga ratusan juta rupiah per bulan secara otomatis** sebagai **Top Recommended Analyst di Ekosistem Aplikasi Traders Family**.

---

## 🎯 1. Pilar Monetisasi & Target Analis Traders Family

```
                           2 MESIN PENDAPATAN RESMI ANALIS
                                          │
          ┌───────────────────────────────┴───────────────────────────────┐
          ▼                                                               ▼
    1. REWARD TF POINT                                          2. REVENUE SUBSCRIBER
 (Konsistensi Kuantitatif Bulanan)                           (Priority Channel VIP Followers)
 ─────────────────────────────────                          ─────────────────────────────────
 • Target: Minimal 300 VP & 5 Settle/Bulan                  • Status: Priority Channel Partner
 • Nilai Konversi: 1 TF Point = Rp 10.000                   • Rating berbasis Sistem Scoring 7-Faktor
 • Potensi: Rp 15jt - Rp 30jt+/bulan                        • Potensi: Rp 50jt - Rp 100jt+/bulan
```

> 📘 **Panduan Lengkap Operasional**: Pelajari detail kualifikasi, regulasi banned, dan blueprint pencairan saldo tunai di [docs/TRADERS_FAMILY_ANALYST_PLAYBOOK.md](docs/TRADERS_FAMILY_ANALYST_PLAYBOOK.md).

---

## 🌟 2. Kapabilitas Baru & Pembaruan Sistem (Update Agustus 2026)

Sistem telah ditingkatkan dengan standar industri kuantitatif mutakhir:

### 1. 📊 Panel Strategy Tester Standar TradingView 2026
Panel bawah interaktif (*Docked Bottom Strategy Tester*) yang mencerminkan standar platform TradingView:
- **Overview Tab**: Bento KPI Cards (*Total Net Profit Pips & VP, Profit Factor, Win Rate %, Max Drawdown, Recovery Factor, Total Closed Trades*).
- **Performance Summary Tab (Tabel Komparasi 3 Kolom)**: Komparasi seimbang antara **All Trades | Long Trades | Short Trades** (*Gross Profit vs Loss, Avg Win vs Avg Loss, Payoff Ratio, Max Consecutive Wins/Losses, Sharpe & Sortino Ratio*).
- **List of Trades Tab (Log Transaksi Interaktif)**: Riwayat order kronologis lengkap dengan waktu masuk/keluar, harga, PnL pips, Valued Pips, status `TP HIT` / `SL HIT`, dan filter cepat *All | Wins | Losses*.
- **Multi-Mode Equity Curve & High-Water Mark Visualizer**:
  - **`📈 Equity Curve`**: Didukung engine `BaselineSeries` TradingView (area hijau Zamrud `#089981` saat surplus profit di atas `0.0 Baseline`, dan merah koral `#f23645` saat defisit modal).
  - **`🏆 Peak & Run-Up`**: Menampilkan kurva ekuitas berdampingan dengan *All-Time High Water Mark* (*Amber Step-Line* `#f59e0b`).
  - **`📉 Underwater Drawdown`**: Grafik kedalaman drawdown murni dari `0.0` ke bawah (`-X pips / -X%`).
  - **`⚡ Interactive Crosshair HUD`**: Floating tooltip saat kursor digeser (*Date, Net Equity, Drawdown %, Peak Record*).

### 2. 🔬 100% Dukascopy Real Historical Feed (Bebas Data Sintetis)
- Seluruh pengujian dan terminal membaca **103.556 Bar H1 data nyata antarbank** dari **Dukascopy Bank SA (Swiss)** sejak 2023 lintas 6 pasangan mata uang utama: `EURGBP`, `USDCHF`, `GBPUSD`, `EURUSD`, `NZDUSD`, `AUDUSD`.
- Generator sintetis telah **dieliminasi 100%** (hanya digunakan pada simulasi acak *Monte Carlo Resampling*).

### 3. 🎲 Monte Carlo 1.000-Path Resampling & Risk of Ruin Engine
- Simulasi 1.000 jalur acak untuk mengukur *Probability of Drawdown > 15%*, *Worst-Case Path*, dan *Confidence Interval (P5, P50, P95)*.

### 4. ⚡ Seamless Dev Runner (`./dev.sh`)
- Skrip satu perintah otomatis yang mengompilasi biner Rust, memvalidasi kesehatan data, dan menjalankan REST API port 5000 + Frontend Svelte 5 port 3000 secara bersih tanpa *orphan process*.

---

## 🧭 3. Evaluasi Jarak Menuju Tujuan Utama (Gap Analysis)

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                        STATUS PENCAPAIAN ROADMAP MONETISASI                                       ║
╠═════════════════════════════════════════════════════╦═══════════════╦═════════════════════════════════════════════╣
║ Pilar & Target Finansial                            ║ Status        ║ Kesiapan Komponen                           ║
╠═════════════════════════════════════════════════════╬═══════════════╬═════════════════════════════════════════════╣
║ 1. Infrastruktur & Compliance Guard (Zero-Penalty)   ║ ✅ 100% SIAP  ║ TfComplianceGuard, Multiplier 4-Tier, Hexa  ║
║ 2. Historical Research Lab & Data Nyata Dukascopy   ║ ✅ 100% SIAP  ║ 103.556 Bar H1, DuckDB, Parquet, Anti-Bias  ║
║ 3. TradingView Visualizer & Strategy Tester Panel   ║ ✅ 100% SIAP  ║ Svelte 5, Runes, Baseline Equity, HUD       ║
║ 4. Monetisasi TF Point (>= 300 VP / Bulan)          ║ 🟡 70% TERCAPAI║ USDCHF profitabel (+211.5 VP), tuning edge  ║
║ 5. Subscriber Priority Channel (7-Faktor Legend)    ║ 🟡 65% TERCAPAI║ Scorecard 28/28 poin, butuh forward test    ║
╚═════════════════════════════════════════════════════╩═══════════════╩═════════════════════════════════════════════╝
```

### 🔍 3 Kesenjangan Utama (*Gaps to Close*):
1. **Penyempurnaan Geometri Stop Loss Pola N**:
   - *Kondisi Saat Ini*: SL pada Bullish Pola N masih ditempatkan di bawah origin $L_1$ (jarak 30-50 pips), sehingga ketika Higher Low $L_2$ patah, akun menahan rugi terlalu lama.
   - *Target Perbaikan*: Memindahkan SL tepat di bawah titik Higher Low $L_2$ (*structural invalidation*). Mengurangi resiko kerugian per trade menjadi hanya 10-15 pips dan mendongkrak rasio R:R menjadi $1:2.5 - 1:3.0$.
2. **Filter Break of Structure (BOS) / Momentum pada Pair Ranging (EURGBP)**:
   - *Kondisi Saat Ini*: Pada pair berkarakter *choppy mean-reverting* (EURGBP), order limit sering kali menangkap pisau jatuh saat breakout baru terjadi.
   - *Target Perbaikan*: Menambahkan filter ADX > 20 atau kemiringan slope EMA 50 untuk mengisolasi entri hanya pada kondisi tren yang memiliki momentum valid.
3. **Forward Testing & Live Autonomous MT5 Stream**:
   - *Kondisi Saat Ini*: Backtest historis multi-tahun telah teruji.
   - *Target Perbaikan*: Menjalankan daemon di Linux server secara 24/5 yang terhubung ke MT5 Bridge untuk forward test live tanpa intervensi.

---

## 🚀 4. Rencana Pengembangan Selanjutnya (Next Milestones)

- [ ] **Milestone 1**: Refaktor kalkulasi Stop Loss ke titik fraktal Higher Low $L_2$ / Lower High $H_2$ pada `formation.rs` guna melipatgandakan Payoff Ratio dan memangkas Max Drawdown.
- [ ] **Milestone 2**: Implementasi indikator ADX / Regime Shift Detector pada `crates/core/domain/src/models/pola_n/strategy.rs`.
- [ ] **Milestone 3**: Integrasi daemon `signal-daemon` dengan akun demo MT5 untuk pencatatan *Forward Testing Journal* harian otomatis ke database TimescaleDB.
- [ ] **Milestone 4**: Deployment VPS Linux (Systemd Service) dan penyiapan webhook notifikasi Telegram VIP.

---

## 🏛️ 5. Arsitektur Software & Prinsip Desain

```
                                    +---------------------------------------+
                                    |            Inbound Adapters           |
                                    |  (CLI / Cron / Axum Web API / TUI)    |
                                    +-------------------+-------------------+
                                                        |
                                                        v [Invokes Inbound Ports]
+---------------------------------------------------------------------------------------------------------+
│                                             HEXAGON CORE                                                │
│                                                                                                         │
│   +-------------------------------------+                +------------------------------------------+   │
│   │         Application Layer           │                │               Domain Layer               │   │
│   │    crates/core/application          │                │            crates/core/domain            │   │
│   │                                     │                │                                          │   │
│   │  - SignalEngineService              │  Evaluates     │  * Pure Models: Candle, Signal, Risk     │   │
│   │  - BacktestService (Modular)        │ -------------> │  * TF Models: TfPairSpec, ValuedPips     │   │
│   │  - StrategyBenchmarkService         │                │  * Strategy: PolaNStrategy, SMC Engine   │   │
│   │  - EdaService (Data Health)         │  Validates     │  * Guard: TfComplianceGuard (Zero Banned)│   │
│   │                                     │ -------------> │  * Outbound Ports (Traits):              │   │
│   │                                     │                │      - MarketDataPort                    │   │
│   │                                     │                │      - SignalPublisherPort               │   │
│   │                                     │                │      - StoragePort                       │   │
│   │                                     │                │      - EconomicCalendarPort              │   │
│   +-------------------------------------+                +------------------------------------------+   │
+---------------------------------------------------------------------------------------------------------+
                                                        |
                                                        | [Implements Outbound Traits]
                                                        v
                                    +---------------------------------------+
                                    |           Outbound Adapters           |
                                    |                                       |
                                    | - publisher-traderfamily (API Client) |
                                    | - notifier-telegram (Multi-Channel)   |
                                    | - storage-db (TimescaleDB / DuckDB)   |
                                    | - dukascopy-rs (Swiss True-Tick)      |
                                    | - broker-connector (MT5 Socket Bridge)|
                                    +-------------------+-------------------+
```

---

## ⚡ 6. Regulasi Resmi Traders Family (Valued Pips Matrix)

$$\text{Valued Pips (VP)} = \text{Pips Bersih} \times \text{Value Multiplier}$$

| Tier | Pasangan Mata Uang | Multiplier | Min Jarak Pending | Min SL & TP | Maks SL & TP | Gap Sinyal 1 ke 2 |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: |
| **Tier 1** | `NZDUSD`, `AUDUSD`, `EURGBP`, `USDCHF` | **2.0x** | 10.0 Pips | 10.0 Pips | 200.0 Pips | **50.0 Pips** |
| **Tier 2** | `USDCAD`, `EURUSD`, `GBPUSD`, `NZDJPY`, `CADJPY`, `AUDJPY` | **1.5x** | 15.0 Pips | 15.0 Pips | 300.0 Pips | **75.0 Pips** |
| **Tier 3** | `CHFJPY`, `USDJPY`, `EURJPY`, `GBPJPY`, `EURNZD` | **1.0x** | 20.0 Pips | 20.0 Pips | 400.0 Pips | **100.0 Pips** |
| **Tier 4** | `XAUUSD` (Gold) | **0.5x** | 30.0 Pips | 30.0 Pips | 500.0 Pips | **100.0 Pips** |

---

## 🚀 7. Panduan Memulai Cepat (Getting Started)

### Quickstart Development Server (One-Command Dev)
```bash
./dev.sh
```
Akses Web UI interaktif TradingView di **http://localhost:3000**.

### Menjalankan Test Suite (Pre-Inspection Gate)
```bash
cargo test --workspace
cd ui && npm run build
```

---

## 📄 Lisensi
Proyek ini dibangun untuk tujuan penelitian kuantitatif, otomasi algoritma, dan edukasi kepatuhan ekosistem Traders Family. Seluruh hak cipta kode dilindungi di bawah lisensi MIT.
