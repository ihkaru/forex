# GEMINI.md - Forex Quantitative Workspace Guidelines

Selamat datang di workspace penelitian kuantitatif, analisis pasar Forex, rewrite scraper, dan otomasi sinyal otomatis untuk monetisasi sebagai **Top Analis di Traders Family Ecosystem**.

---

## 🎯 Objektif Utama & Strategi Finansial (Monetization Engine)

Tujuan utama workspace ini adalah **menghasilkan pendapatan puluhan hingga ratusan juta rupiah per bulan secara otomatis** melalui 2 pilar resmi Traders Family:
1. **Reward TF Point Bulanan**: Menghasilkan minimal **300 Valued Pips (VP)** dan $\ge 5$ sinyal settled per bulan untuk dikonversikan menjadi dana tunai rupiah (1 Point = Rp 10.000).
2. **Revenue Sharing Subscriber (Priority Channel)**: Mempertahankan metrik performa sempurna pada **Sistem Scoring 7-Faktor TF (Juli 2026)** guna memaksimalkan ranking, harga channel, dan jumlah pengikut berbayar.

---

## 🏛️ Arsitektur & Prinsip Desain Utama

Repo ini mengadopsi standar rekayasa software modern tingkat lanjut:

### 1. Hexagonal Architecture (Ports & Adapters)
- **Domain (`crates/core/domain`)**: Jantung sistem. Bebas dari ketergantungan eksternal (zero I/O). Berisi entitas, value object, kalkulasi matematika, model `PolaNStrategy`, spesifikasi regulasi `TfPairSpec`, invariant validator `TfComplianceGuard`, dan **Ports (Traits)**.
- **Application (`crates/core/application`)**: Orkestrator *use case* (misal: `SignalEngineService`, `BacktestService`).
- **Adapters (`crates/adapters/*`)**: Implementasi konkret dari Ports untuk I/O (Database TimescaleDB, REST/WebSocket, Broker MT5/cTrader, Publisher Trader Family, Notifier Telegram).
- **Scrapers (`crates/scrapers/*`)**: Library hasil reverse engineering dan rewrite ke Rust untuk penarikan data publik (Dukascopy True-Tick, ForexFactory, Myfxbook, Investing).
- **Apps (`apps/*`)**: Binary entrypoints (CLI, Background Daemons, API Server).

### 2. Interface-First (Trait-Driven Development)
- **Definisikan Kontrak Terlebih Dahulu**: Setiap kapabilitas (penarikan harga, publikasi sinyal, penyimpanan histori) **wajib** dibuatkan trait-nya di `crates/core/domain/src/ports/` sebelum menulis adapter konkret.
- Memungkinkan *mocking* instan untuk unit test strategi tanpa perlu koneksi internet atau database nyata.

### 3. Composition over Inheritance
- Rust secara alami tidak memiliki pewarisan kelas (*inheritance*).
- Setiap strategi trading disusun dengan menggabungkan komponen modular (*composition*):
  ```rust
  // KOMPOSISI STRATEGI POLA N
  pub struct PolaNStrategy {
      pub swing_detector: SwingPointDetector,
      pub formation_engine: PolaNFormationEngine,
  }
  ```

### 4. Dependency Injection Pattern (Pure DI / Composition Root)
- **Hindari Global State / Singleton**: DILARANG menggunakan global static mutable state (`lazy_static!`, `static mut`) untuk client jaringan atau database.
- **Constructor Injection**: Semua service di level `application` menerima dependensi melalui constructor `new(...)` berupa trait object (`Arc<dyn Port>`) atau generic trait bounds (`<P: SignalPublisherPort>`).
- **Composition Root di Layer Binary (`apps/*`)**: Pengkabelan (wiring) antara adapter konkret dan engine dilakukan secara terpusat di `main.rs` aplikasi:
  ```rust
  // COMPOSITION ROOT (apps/signal-daemon/src/main.rs)
  let broker = Arc::new(BrokerConnector::new(...));
  let tf_publisher = Arc::new(TraderFamilyPublisher::new(...));
  let storage = Arc::new(SqlxStorage::connect(...).await?);
  
  let engine = SignalEngineService::new(broker, vec![tf_publisher], storage, strategies, risk);
  ```

---

## ⚡ Regulasi Resmi Traders Family (Valued Pips & 4-Tier Constraints)

Setiap modul pembuat sinyal **wajib** mematuhi invariant Traders Family yang dikawal oleh `TfComplianceGuard`:

### 1. Sistem Valued Pips (VP)
$\text{Valued Pips} = \text{Pips Bersih} \times \text{Value Multiplier}$.
* **Tier 1 (2.0x)**: `NZDUSD`, `AUDUSD`, `EURGBP`, `USDCHF` (Min SL/TP 10 pips, Max SL/TP 200 pips, Gap 50 pips).
* **Tier 2 (1.5x)**: `USDCAD`, `EURUSD`, `GBPUSD`, `NZDJPY`, `CADJPY`, `AUDJPY` (Min SL/TP 15 pips, Max SL/TP 300 pips, Gap 75 pips).
* **Tier 3 (1.0x)**: `USDJPY`, `EURJPY`, `GBPJPY`, `CHFJPY`, `EURNZD` (Min SL/TP 20 pips, Max SL/TP 400 pips, Gap 100 pips).
* **Tier 4 (0.5x)**: `XAUUSD` / Gold (Min SL/TP 30 pips, Max SL/TP 500 pips, Gap 100 pips).

### 2. Invariant Kepatuhan Sinyal (Zero-Penalty Guarantee)
1. **Wajib Pending Order**: Hanya `BuyLimit`, `SellLimit`, `BuyStop`, `SellStop` (Instant execution otomatis ditolak).
2. **Batas Rasio Risk:Reward**: Wajib $1:1.0 \le R:R \le 1:3.0$ (DILARANG melebihi 1:3.0).
3. **Maksimal Stop Loss**: $\text{SL} \le 1.5 \times \text{Take Profit}$.
4. **Batas Sinyal per Pair**: Maksimal 2 sinyal aktif per pair.
5. **Durasi Kadaluwarsa**: 1–48 jam (96 jam jika dibuat hari Jumat).
6. **No Intervention**: Sinyal yang berstatus Running tidak boleh dimodifikasi SL/TP-nya.

### 3. Matriks 7 Pilar Penentu Skor Channel (Update Juli 2026)
* **Recovery Factor (23.53%)**: $\text{Nett P/L} / \text{Max DD} \ge 8.0$ (Skor 4 Poin).
* **Level Channel (17.65%)**: Legend (Skor 4 Poin) / Master (2 Poin).
* **Status Kemitraan (17.65%)**: Priority Channel (Skor 4 Poin) / Prospect (2 Poin).
* **Profit Factor (17.65%)**: $PF \ge 2.10$ 6 bulan terakhir (Skor 4 Poin).
* **Monthly Loss Ratio (11.76%)**: $0\% - 10\%$ dari rata-rata profit (Skor 4 Poin).
* **Profit Months (5.88%)**: 6 dari 6 bulan berturut-turut profit (Skor 4 Poin).
* **Subscriber (5.88%)**: $\ge 501$ Subscriber Berbayar (Skor 4 Poin).

---

## 🌐 Aturan Limitasi Sumber Data & Pencegahan Bias (Anti-Bias Rules)

1. **🔬 Riset & Backtesting Multi-Dekade (Historical Lab)**:
   - **Sumber Wajib**: **Dukascopy Bank SA (Swiss)** via `crates/scrapers/dukascopy-rs`.
   - **Karakteristik**: True Tick-by-Tick murni antarbank sejak 2003 (20+ tahun), gratis tanpa API Key, dikompresi LZMA `.bi5` ke Apache Parquet, dianalisis instan via DuckDB.
   - **PANTANGAN MUTLAK**: DILARANG melakukan backtest panjang hanya mengandalkan riwayat bawaan terminal MT5 ritel, karena batas kedalaman bar (shallow history) dan *pseudo-tick interpolation* menyebabkan bias *curve-fitting*.

2. **🎯 Live Autonomous Streaming & Forward Testing**:
   - **Sumber**: **MetaTrader 5 EA Bridge** (`crates/adapters/broker-connector/mql5/ForexHexagonBridge.mq5`) + **cTrader Open API ProtoBuf** (`CtraderOpenApiConnector`).
   - **Karakteristik**: Sub-millisecond local socket/TCP stream yang mencerminkan spread dan eksekusi riil broker subscriber.

---

## ⚡ Protokol Integritas & Kualitas Data Pasar (Data Integrity #1)

1. **Normalisasi Timezone ke UTC**: Seluruh timestamp data feed (MetaTrader/Dukascopy/cTrader) **wajib dinormalkan ke UTC** (`chrono::DateTime<Utc>`).
2. **Spread Anomaly & Rollover Spike Filter**: Setiap tick wajib divalidasi oleh `DataIntegrityValidator`. Tick dengan $Ask < Bid$ (data korup) atau $\text{Spread} > 5.0\text{ pips}$ ditolak otomatis.
3. **Presisi Finansial Murni**: DILARANG menggunakan `f64` untuk harga, saldo, lot, pips, SL/TP. **WAJIB** menggunakan `rust_decimal::Decimal`.
4. **Candle Mathematical Invariants**:
   - $High \ge Low$, $High \ge Open$, $High \ge Close$
   - $Low \le Open$, $Low \le Close$
   - $Volume \ge 0$

---

## 🗄️ Arsitektur Database: Tiered Hybrid Storage

1. **Tier 1 (Hot State / In-Memory)**: `InMemoryStorage` untuk evaluasi sinyal real-time sub-mikrodetik dan unit test mock.
2. **Tier 2 (Analytical OLAP / Parquet)**: `DuckDbAnalyticalEngine` untuk membaca file `.parquet` data historis 20 tahun dalam hitungan milidetik.
3. **Tier 3 (Transactional OLTP / TimescaleDB)**: `SqlxStorage` dengan skema migrasi [migrations/001_initial_schema.sql](file:///home/ihza/Projects/forex/crates/adapters/storage-db/migrations/001_initial_schema.sql) untuk persistensi sinyal, pesanan broker, dan log publikasi Trader Family.

---

## 🎨 Arsitektur Frontend: Hexagonal Architecture, Svelte 5 & Tailwind CSS v4 (Standar 2026)

Frontend di direktori [`ui/`](ui/) mengadopsi standar modern **Hexagonal Architecture (Ports & Adapters)** berpadu dengan **Feature-Sliced Design (FSD)** berbasis **TypeScript**:

### 1. Prinsip Desain Rekayasa Perangkat Lunak Frontend
1. **Interface-First Design (Kontrak Port Terlebih Dahulu)**:
   - Setiap kapabilitas eksternal (penarikan data pasar, eksekusi sinyal, audit EDA, pembacaan backtest) **wajib** dibuatkan kontrak TypeScript `interface`-nya di `ui/src/ports/` (`IMarketDataPort`, `ISignalPublisherPort`, `IBacktestPort`, `IEdaHealthPort`) sebelum menulis adapter konkret.
   - Memungkinkan pengujian unit terisolasi (mocking) tanpa perlu server backend aktif.
2. **Composition over Inheritance**:
   - DILARANG menggunakan hierarki pewarisan class bertingkat (*inheritance*).
   - Seluruh fungsionalitas UI dan service disusun melalui komposisi objek modular (*composition*):
     ```typescript
     export class TradingViewChartAdapter {
       constructor(
         private readonly containerId: string,
         private readonly marketDataPort: IMarketDataPort
       ) {}
     }
     ```
3. **Pure Dependency Injection (Constructor Injection & Composition Root)**:
   - DILARANG menggunakan singleton atau global mutable state (`window.appState = ...`).
   - Semua dependensi diinjeksikan melalui constructor `new Adapter(port)`.
   - Seluruh pengkabelan dependensi dilakukan secara terpusat di `ui/src/index.ts` (**Composition Root**).
4. **TradingView Layering System via Interface (`IChartLayer`)**:
   - Visualisasi teknikal (Trades, Fractal Swings, Dual EMA, Live Signal Overlays) disusun modular melalui kontrak `IChartLayer` di `ui/src/ports/layers.ts`.
   - `ChartLayerManager` mengkomposisikan berbagai layer secara terpisah sehingga setiap layer dapat di-toggle (Show/Hide) secara independen tanpa merusak performa rendering WebGL.
5. **Zero-Dependency Domain Core**:
   - Layer domain di `ui/src/domain/` murni berisi matematika trading (`Candle`, `Signal`, `TfPairSpec`, `TfComplianceGuard`) tanpa dependensi ke framework UI, `fetch`, atau DOM.

### 2. Teknologi & Library Terpilih (2026)
* **Framework**: **Svelte 5 (Runes: `$state`, `$derived`, `$props`) + Vite** untuk performa *Zero Virtual DOM* dan *surgical sub-millisecond DOM updates* saat live tick streaming.
* **Styling & Design System**: **Tailwind CSS v4** dengan arsitektur *CSS-First Configuration* via `@theme` (tanpa `tailwind.config.js`, didukung Rust Oxide engine, dan ruang warna `oklch()`).
* **UI Components (Full Ownership)**: **`shadcn-svelte`** (Komponen headless berbasis runes-native) & **`Tremor / Tremor Raw`** (Bento KPI Cards, Progress Rings, Sparklines, Delta Badges).
* **Charting Engine**: **`TradingView Lightweight Charts (v4.2 / v5+)`** — Engine charting HTML5 Canvas/WebGL 120 FPS dengan custom trade overlay primitives (Entry, SL, TP R:R bounding box).
* **Iconography**: **`Lucide Icons`** — Sistem ikonografi vektor finansial ultra-ringan.

---

## 🔬 Metodologi Kuantitatif & Protokol Pencegahan 7 Jebakan Fatal Backtest (Anti-Pitfalls 2026)

Untuk mencegah ilusi profit palsu (*too good to be true*) dan menjamin validitas riset kuantitatif di pasar nyata, setiap simulasi backtest di `crates/core/application/src/services/backtest.rs` **wajib** mematuhi 7 invariant anti-bias:

1. **Anti Look-Ahead Bias (Rolling Window Bar-by-Bar)**:
   - Keputusan pada bar $i$ HANYA boleh membaca slice historis yang sudah tertutup $\le i-1$.
   - DILARANG menggunakan high/low/close bar saat ini sebagai konfirmasi sebelum bar selesai.

2. **Pending Order Realistic Fill & Expiration Lifecycle**:
   - Status pesanan tunda (`BuyLimit`/`SellLimit`) berstatus `Pending` dan HANYA menjadi `Filled` jika harga pasar ($Ask/Bid$ + spread) menyentuh level limit.
   - Jika dalam 24 jam order tidak terjemput, order otomatis dibatalkan (**`Expired` / 0 PnL**). DILARANG menganggap pending order langsung aktif seketika (*Instant Fill Bug*).

3. **Intrabar Ambiguity & Conservative Worst-Case Resolution**:
   - Jika dalam 1 bar candle yang sama harga menyentuh level Take Profit dan Stop Loss sekaligus, simulator **WAJIB menganggap Stop Loss terkena terlebih dahulu**.

4. **Realistic Transaction Costs & Dynamic Spread Injection**:
   - Setiap simulasi wajib menyertakan markup spread realistis ($\ge 1.2\text{ pips}$ pada major pair) yang mencerminkan eksekusi broker ritel subscriber (*MRG Mega Berjangka*).

5. **Anti Overfitting (Walk-Forward & Out-of-Sample Validation)**:
   - Dataset dibagi menjadi **70% In-Sample** (untuk eksplorasi parameter) dan **30% Out-of-Sample** (evaluasi buta tanpa tuning).
   - Logika strategi diutamakan berbasis geometri fraktal sederhana (*Parsimony Rule*) daripada tumpukan puluhan indikator kurva.

6. **Anti P-Hacking & Multi-Pair 4-Tier Cross Validation**:
   - DILARANG melakukan *cherry-picking* pada 1 pair yang kebetulan profit tinggi. Strategi wajib diuji lintas 6 pasang mata uang dari Tier 1 hingga Tier 4.

7. **Strict Platform Rule Invariant (No-Intervention Guarantee)**:
   - Sesuai regulasi Traders Family, posisi yang berstatus *Running* tidak boleh diintervensi (dilarang trailing manual / BEP parsial). Exit wajib fixed deterministik pada TP atau SL.

---

## 🛡️ Checklist Kepatuhan & Standar Rekayasa Kode (Engineering Standards & Compliance Checklist)

Setiap penambahan fitur, refaktor, atau adapter baru **wajib** mematuhi 8 langkah invariant berikut:

```
[ ] 1. PORT FIRST          : Tulis Trait di crates/core/domain/src/ports/ atau ui/src/ports/ terlebih dahulu.
[ ] 2. DECIMAL PRECISION   : Gunakan rust_decimal::Decimal untuk seluruh kalkulasi harga dan uang.
[ ] 3. COMPOSITION OVER INH: Komposisikan kapabilitas melalui constructor injection (Pure DI), tanpa inheritance.
[ ] 4. CODE SIZE LIMIT     : DILARANG membuat 1 file melebihi 600 baris kode. Pecah file >600 baris secara modular.
[ ] 5. TF COMPLIANCE       : Pastikan seluruh sinyal lolos validasi TfComplianceGuard (Pending Limit, RR 1:1-1:3).
[ ] 6. ANTI-BIAS AUDIT     : Pastikan backtest mematuhi 7 Protokol Anti-Pitfalls (No lookahead, realistic fill, intrabar worst-case).
[ ] 7. PRE-INSPECTION GATE : WAJIB menjalankan cargo test --workspace dan npm run build SEBELUM diperiksa user secara manual.
[ ] 8. STATIC AUDIT        : Wajib 0 error, 0 test failure, dan 0 build warning.
```

---

## 📏 Aturan Batas Maksimal Panjang File (< 600 Baris)

- **Prinsip Single Responsibility & Modularity**: Tidak boleh ada file monolitik yang melebihi **600 baris**.
- **Strategi Pemecahan (Decomposition)**:
  1. Pisahkan layer Domain Models, Value Objects, dan Enums ke sub-file tersendiri.
  2. Pisahkan Router Handlers dan HTTP DTOs ke folder `handlers/` terdedikasi.
  3. Pisahkan Unit Test suite yang panjang ke file `tests.rs` atau direktori `tests/`.
  4. Satukan dan orkestrasikan modul menggunakan **Composition over Inheritance** dan **Dependency Injection** pada Composition Root (`main.rs`).

