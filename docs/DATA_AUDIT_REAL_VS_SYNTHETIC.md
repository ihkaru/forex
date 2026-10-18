# 🔬 Laporan Audit Forensik Kualitas Data Pasar: Real Broker MT4 vs Dukascopy vs Sintetis

Dokumen ini mencatat secara komprehensif hasil investigasi, audit integritas, serta penataan ulang arsitektur market data pada sistem Hexagon Quantitative Engine — khususnya mengatasi bias data sintetis/demo, standardisasi ticker broker riil, serta adopsi resolusi tick berpresisi tinggi.

---

## 📌 1. Latar Belakang & Temuan Audit (Forensic Discovery)

Dalam audit pra-rilis sistem quant 2026, ditemukan beberapa anomali kritis pada pipeline market data yang berpotensi menimbulkan **False Edge** (ilusi performa tinggi pada backtest namun gagal saat forward testing akun riil):

1. **Ketidaksesuaian Provenance Akun (Real vs Demo)**:
   - Akun MetaTrader 4 yang terhubung pada workstation adalah **Akun Riil MaxrichGroup** (`Server: MaxrichGroup-Real`, Login `89496278`).
   - Namun, bridge dan routing sebelumnya secara default menandai payload dengan tag `MrgDemoMt4` atau fallback generik `DukascopyEcn`.
   - Dampak: Integritas provenance pasar tercemar, menyulitkan pemisahan antara data pasar live broker sebenarnya dengan data demo.

2. **Truncation Ticker Simbol Broker**:
   - Broker MaxrichGroup / Exness menggunakan sufiks simbol akun standar/cent/ECN seperti `XAUUSD_MRG`.
   - Normalisasi kode sebelumnya memotong sufiks tersebut menjadi `XAUUSD` secara prematur, menyebabkan data historis broker tertimpa atau bentrok dengan data Dukascopy.

3. **Ketiadaan Parameter Kritis Struktur Mikro Pasar (Market Microstructure Gaps)**:
   - Data candle historis standar hanya mencatat 5 dimensi OHLCV (`open`, `high`, `low`, `close`, `volume`).
   - Parameter krusial penentu profitabilitas riil tidak terekam:
     - **Spread Dinamis**: Melebar saat sesi pergantian hari (rollover 21:00-23:00 GMT) dan rilis berita ekonomi *high impact* (NFP, CPI, FOMC).
     - **Real Volume**: Volume transaksi lot riil institusional vs tick volume (frekuensi kuotasi).
     - **Spesifikasi Kontrak Broker (`BrokerSymbolSpec`)**: Nilai tick (`tick_value`), ukuran kontrak (`contract_size`), *swap long/short*, aturan *triple swap* hari Rabu, *stop level*, dan *freeze level*.

4. **Intrabar Ambiguity pada Resolusi Bar H1**:
   - Pada bar H1, tidak diketahui urutan kronologis terjadinya High vs Low.
   - Contoh: Jika suatu bar menyentuh Stop Loss dan Take Profit sekaligus, evaluasi bar statis mengandalkan asumsi arbitrer yang bisa menyebabkan *Lookahead Bias* atau *Optimism Bias*.

---

## 🧱 2. Perbandingan Karakteristik 3 Sumber Data

| Dimensi | Data Sintetis (Mock/Sim) | Dukascopy Bank ECN | Live Broker MT4 (Exness / MRG Real) |
|:---|:---|:---|:---|
| **Ketersediaan** | Instan / Tak Terbatas | Historis Panjang (2015–2026) | Real-time & Recent (500–1.000 bar) |
| **Karakteristik Spread** | Statis / Ditetapkan manual | Floating ECN (Sangat Tipis, 0.5–1.5 pips) | Floating Broker Retail + Markup (15–45 pts) |
| **Slippage & Requote** | Nol (Ideal 100%) | Nol pada bar statis | Riil tergantung likuiditas & pending order |
| **Simbol Asli** | `EURUSD`, `XAUUSD` | `EURUSD`, `XAUUSD` | `XAUUSD_MRG`, `EURUSD_MRG`, dll. |
| **Data Tick** | Dihasilkan matematis | Tersedia format `.bi5` (terkompresi lzma) | Kuotasi live millisecond dari server broker |
| **Kegunaan Utama** | Unit test arsitektur, Monte Carlo | Backtest makro 10 tahun (pembuktian edge) | Forward test, execution fill, validasi live |

---

## 🛠️ 3. Solusi Arsitektural yang Diimplementasikan

### A. Strict Hexagonal Domain Isolation & Models
Model `Candle` dan `Tick` pada [`crates/core/domain/src/models/candle.rs`](../crates/core/domain/src/models/candle.rs) diperkaya dengan field opsional non-breaking (`Option<T>`):
```rust
pub struct Candle {
    pub symbol: Symbol,
    pub timeframe: Timeframe,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub source: MarketDataSource,
    pub spread: Option<Decimal>,       // Spread riil dalam unit harga
    pub real_volume: Option<Decimal>,  // Volume lot riil jika didukung broker
}
```
Field `Option` memastikan sumber non-broker (seperti Dukascopy atau agregator REST) tetap valid tanpa merusak kontrak publik domain.

### B. Otomasi Ekstraksi Broker Contract Spec
Model baru `BrokerSymbolSpec` ([`crates/core/domain/src/models/broker_spec.rs`](../crates/core/domain/src/models/broker_spec.rs)) menangkap spesifikasi simbol langsung dari fungsi MQL4 `MarketInfo()`:
- `digits`, `point`, `tick_value`, `tick_size`, `contract_size`
- `swap_long`, `swap_short`, `swap_rollover_3days` (Triple swap hari Rabu)
- `stop_level_pts`, `freeze_level_pts`, `min_lot`, `lot_step`, `max_lot`, `margin_required`

### C. Dual-Resolution Hybrid Backtesting Engine
Engine [`BacktestService`](../crates/core/application/src/services/backtest/service.rs) menerapkan pola eksekusi adaptif:
1. **Mode 1: Tick-Walk Resolution (Zero-Ambiguity)**:
   Jika data tick historis untuk interval bar tersebut tersedia di database / storage port, engine menjalankan simulasi kronologis per-tick.
   - Pending order terisi tepat saat kuotasi `Ask`/`Bid` menyentuh harga.
   - SL dan TP diperiksa dengan harga kuotasi nyata, mengeliminasi asumsi urutan intrabar.
2. **Mode 2: Candle-Level Fallback Resolution**:
   Jika data tick belum ada (misal data 10 tahun Dukascopy), engine mundur secara anggun ke evaluasi bar H1 dengan `effective_spread` dinamis (menggunakan `candle.spread` jika tersedia, atau spread offset konservatif).

### D. Penyimpanan Pasar Terpadu (Unified Embedded SQLite WAL Storage)
Arsitektur persistensi produksi utama menggunakan [`SqliteStorage`](../crates/adapters/storage-db/src/sqlite.rs):
- **Embedded & Zero-Network Latency**: Menggunakan driver SQLite dengan mode WAL (`PRAGMA journal_mode=WAL`), indexing B-Tree teroptimasi untuk query rentang `(symbol, timeframe, timestamp, source)`.
- **Atomic Batch Ingestion**: Ingestion ratusan ribu candle dan tick dalam batch transaksi atomic tanpa membebani disk I/O.
- **Skrip Migrasi Opsional Postgres/TimescaleDB**: Skrip [`002_enriched_candles_ticks_specs.sql`](../crates/adapters/storage-db/migrations/002_enriched_candles_ticks_specs.sql) tetap tersedia sebagai skema referensi opsional jika di masa mendatang diekspansi ke PostgreSQL/TimescaleDB eksternal.

### E. Preservasi Ticker Asli Broker & Pembersihan Aman
- Ticker asli broker (misal `XAUUSD_MRG`) dipreservasi dalam parsing raw bridge.
- Endpoint pembersihan data (`POST /api/market/reset` & `DELETE /api/market/candles`) dilindungi kunci otentikasi header `X-Reset-Key` atau parameter `key=...` guna mencegah manipulasi tidak sengaja di production.

---

## 📊 4. Status Observasi Data Live & Siklus Pasar

### Mengapa Data Tick Belum Masuk di Akhir Pekan?
Pada pengujian hari **Sabtu, 17 Oktober 2026**:
1. **Status Koneksi**: EA MetaTrader 4 `ForexHexagonBridge` sukses terhubung ke server Rust via TCP port 5555.
2. **Spesifikasi Simbol (`SPEC`)**: Berhasil ditransmisikan dan tersimpan untuk seluruh pair Market Watch.
3. **Bar Historis H1 (`BAR`)**: 500 bar historis per simbol berhasil di-ingest dengan tag `MrgRealMt4`.
4. **Data Tick Live (`TICK`)**:
   - Di pasar Forex & Gold spot global, bursa tutup pada hari **Jumat pukul 22:00 GMT** dan baru buka kembali pada hari **Minggu pukul 22:00 GMT**.
   - Event `OnTick()` di MetaTrader hanya dipicu saat ada kuotasi harga baru yang diterima dari server broker.
   - Selama akhir pekan saat pasar tutup, broker tidak memancarkan tick baru. Begitu pasar buka kembali pada sesi Asia, live tick stream akan mengalir otomatis dan diserap oleh `TickStoragePort`.

---

## 🔬 5. Analisis Perbandingan Menyeluruh: Dukascopy ECN vs MT4 Real Broker

Berikut adalah analisis mendalam mengenai karakteristik pembeda antara dataset historis Dukascopy ECN dan live feed MT4 MaxrichGroup:

| Parameter Evaluasi | Dukascopy Bank SA (ECN Swiss) | MetaTrader 4 (MaxrichGroup-Real) | Implikasi Kuantitatif |
|:---|:---|:---|:---|
| **Horizon Waktu (Panjang Sampel)** | **10 Tahun Penuh (2015–2026)** (~68.106 bar H1 XAUUSD) | **~500 Bar Terakhir** (~20 hari bursa berjalan) | Dukascopy digunakan untuk *Macro Walk-Forward Validation* & *Multi-Year Regime Shift*, sedangkan MT4 untuk *Microstructure Execution Calibration*. |
| **Karakteristik Spread** | ECN Raw Spread Sangat Tipis (0.5 – 1.5 pips pada EURUSD; 1.0 – 2.5 pips pada Gold) | Retail Broker Floating Spread + Markup (15 – 45 pips pada Gold) | Backtest pada MT4 memperhitungkan biaya spread retail riil yang menguji ketahanan strategi terhadap pelebaran spread. |
| **Biaya Inap (Overnight Swap)** | Model interbank rollover tanpa markup | Spesifikasi broker retail riil (termasuk bunga kredit/debit broker) | MT4 menerapkan penalti swap riil dengan pengali 3x (*triple swap*) di malam Rabu. |
| **Stop Level & Freeze Level** | Tidak ada batasan jarak pending order (0 pts) | Dibatasi oleh broker (`stop_level_pts = 30-50 pts`) | Pada akun MT4, pending limit order yang terlalu dekat dengan market price akan ditolak (`ERR_INVALID_STOPS`). Engine quant memvalidasi ini via `enforce_stop_levels`. |
| **Basis Zona Waktu (Timestamp)** | **UTC / GMT Murni** | **Server Broker (GMT+3 / GMT+2 DST)** | Jika tidak dinormalisasi, timestamp MT4 akan bergeser +3 jam. Telah dinormalisasi via EA: `bar_time = iTime() - (TimeCurrent() - TimeGMT())`. |
| **Tujuan & Peran dalam Sistem** | Pembuktian *Statistical Edge*, kestabilan matematis, dan ketahanan drawdown jangka panjang. | Kalibrasi mikro-struktur, audit slippage, validasi ketaatan aturan Traders Family, dan eksekusi forward live. |

---

## 🧭 6. Rekonstruksi Mekanisme Realistis Eksekusi Backtesting

Untuk memastikan tidak ada bias optimisme (*Optimism Bias*) maupun asumsi eksekusi yang naif, engine [`BacktestService`](../crates/core/application/src/services/backtest/service.rs) telah dilengkapi 3 pilar realisme broker:

### A. Penyelesaian Masalah "Cerita Siapa Lebih Dulu" (*Intrabar Ambiguity Resolution*)
Pada bar candlestick H1, urutan pencapaian harga ekstrem (apakah harga High disentuh lebih dulu atau Low lebih dulu) menentukan apakah posisi terkena Take Profit (TP) atau Stop Loss (SL) ketika kedua batas harga berada di dalam rentang bar yang sama.
- **Pendekatan Lama**: Asumsi acak atau bias TP-first yang menghasilkan hasil fiktif yang menyenangkan (*flattering bias*).
- **Pendekatan Baru: Directional 4-Point Intrabar Pathing**:
  - **Candle Bullish** (`close >= open`): Dominasi pergerakan harga dimulai dari pembukaan, menguji titik terendah (dip), reli ke puncak, lalu ditutup di bawah puncak.
    - Urutan Lintasan: `Open ➔ Low ➔ High ➔ Close`.
    - Implikasi: Pending Buy terjemput pada saat dip ke `Low`. Jika pergerakan berlanjut menembus SL ke bawah, SL terpicu sebelum harga sempat reli ke High. Sebaliknya, posisi Sell terancam menyentuh TP pada `Low` sebelum reli ke High.
  - **Candle Bearish** (`close < open`): Dominasi pergerakan harga dimulai dari dorongan awal ke atas (rally), penolakan di puncak, penurunan tajam ke titik terendah, lalu ditutup sedikit di atas titik terendah.
    - Urutan Lintasan: `Open ➔ High ➔ Low ➔ Close`.
    - Implikasi: Pending Sell terjemput pada saat reli ke `High`. Jika pergerakan rally melampaui SL ke atas, SL Sell terpicu sebelum harga turun ke Low. Sebaliknya, posisi Buy berkesempatan menyentuh TP pada `High` sebelum penurunan ke Low.
  - **Mode Resolusi Super-Tinggi (Tick-Walk)**: Jika data tick dari MT4 tersedia, engine beralih ke simulasi kronologis mikrodetik yang 100% bebas ambiguitas bar.

### B. Pemotongan Biaya Inap Overnight Swap (Termasuk Triple Swap Hari Rabu)
- Posisi yang ditahan melewati pergantian hari bursa (22:00 GMT / 00:00 Waktu Server MT4) dikenakan biaya swap per hari:
  $$\text{Swap Pips} = \text{broker\_spec.swap\_long/short} \times \text{hari\_inap}$$
- Khusus pada hari **Rabu malam** (pergantian hari Rabu ke Kamis GMT), posisi dikenakan **Triple Swap (3x)** untuk mengompensasi penyelesaian transaksi akhir pekan (T+2 settlement rule perbankan internasional).
- Total swap yang dipotong dicatat secara transparan pada laporan backtest via field `total_swap_deducted_pips` dan ditampilkan pada panel UI.

### C. Penegakan Jarak Stop Level Broker (*ERR_INVALID_STOPS Prevention*)
- Pada broker retail, pending limit order tidak dapat ditempatkan sembarangan jika jaraknya lebih dekat dari `broker_spec.stop_level_pts` terhadap harga pasar saat ini.
- Fitur `enforce_stop_levels: true` pada `BacktestConfig` menolak order yang melanggar ketentuan ini, memastikan strategi hanya menghasilkan sinyal yang legal dieksekusi di platform MT4.

---

## ⚖️ 7. Forensic & Resolusi Discrepancy UI vs Backtesting Engine

### Investigasi Perbedaan Angka Pips pada Tampilan UI:
1. **Penyebab Discrepancy Awal**:
   - Komponen `TopBentoBar` di bagian atas dashboard membaca endpoint `/api/audit/full` yang menjalankan `QuantAuditService` dengan baseline `PolaNStrategy` v1 lintas 7 pair forex (menghasilkan angka agregat portfolio `3.262,5 VP`).
   - Sementara itu, dropdown pemilih strategi menampilkan label model spesialis `pola-n-v8` Titan Emas dengan rekor 10-tahun `+12.416,5 VP` (1.390 trades, RF 12.18, PF 1.63).
   - Akibatnya, terjadi perbedaan angka antara badge strategi dan angka Valued Pips di kartu header Bento Bar.
2. **Solusi yang Diimplementasikan**:
   - `App.svelte` dimutakhirkan agar memprioritaskan hasil eksekusi backtest aktual dari strategi dan data source yang sedang aktif (`composition.backtestPort.runBacktest(selectedStrategyId, selectedMarketSource)`).
   - Ketika pengguna memilih strategi `pola-n-v8` atau beralih antara data source `dukascopy` vs `mrg_real`, callback `loadBacktestAndScorecard()` dieksekusi secara reaktif sehingga metrik `valuedPips`, `totalTrades`, `wferPct`, dan `isTfQualified` di Bento Bar, Strategy Tester, dan Signal HUD selalu sinkron dan konsisten.
   - Panel Strategy Tester kini menampilkan badge model eksekusi (`DIRECTIONAL_4POINT_INTRABAR_PATHING` / `DUAL_RESOLUTION_HYBRID`), badge sumber data (`MT4 REAL` vs `DUKASCOPY`), serta kartu metrik `Swap Rollover` (biaya inap riil).

---

## 🎯 8. Kesimpulan & Rekomendasi Eksekusi
1. **Uji Validitas Jangka Panjang (Macro Regime Shift)**:
   Gunakan dataset Dukascopy ECN (2015–2026, 68.000+ bar) dengan Directional 4-Point Intrabar Pathing untuk memastikan ketahanan strategi menghadapi anomali makro (COVID 2020 crash, inflasi global 2022, rally all-time-high emas 2024–2026).
2. **Kalibrasi Mikro-Struktur & Forward Testing**:
   Gunakan live feed MT4 MaxrichGroup (`MrgRealMt4`) untuk memverifikasi eksekusi pada kondisi spread floating 20–45 pips, menguji pemotongan swap harian, dan memastikan ketaatan terhadap aturan pending limit order Traders Family.
3. **Integritas Tanpa Kompromi**:
   Sistem sepenuhnya menolak data sintetis dalam evaluasi performa, menyajikan angka apa adanya (*zero flattering bias*), serta mencatat bukti terminal kuantitatif secara deterministik.
