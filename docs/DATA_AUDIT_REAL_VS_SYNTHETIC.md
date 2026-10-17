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

### D. Penyimpanan Tick Berperforma Tinggi (TimescaleDB Gorilla Compression)
Skrip migrasi database [`002_enriched_candles_ticks_specs.sql`](../crates/adapters/storage-db/migrations/002_enriched_candles_ticks_specs.sql):
- Mengonversi tabel `ticks` menjadi hypertable TimescaleDB.
- Menerapkan algoritma kompresi **Gorilla / Delta-of-Delta** untuk timestamp dan harga `Bid`/`Ask`.
- Mereduksi konsumsi disk hingga 90-95%, menyamai efisiensi format columnar Parquet/Vortex.

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

## 🎯 5. Kesimpulan & Rekomendasi Pengujian
- **Backtesting Historis Jangka Panjang (5-10 Tahun)**: Tetap menggunakan dataset Dukascopy ECN yang bebas bias lookahead dan spread konsisten.
- **Validasi Eksekusi & Forward Testing**: Menggunakan data feed MT4 `MrgRealMt4` untuk menguji ketahanan slippage, spread dinamis, dan ketaatan regulasi pending limit order Traders Family.
