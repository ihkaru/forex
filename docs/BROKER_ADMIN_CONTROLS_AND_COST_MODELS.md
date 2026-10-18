# 🏛️ Katalog Kendali Admin MetaTrader & Model Biaya Transaksi Riil

Dokumen ini mendokumentasikan seluruh variabel operasional, parameter eksekusi, dan struktur biaya yang dapat dikendalikan oleh administrator broker MetaTrader (MT4/MT5 Server Manager & Virtual Dealer Plugin), membedakan fakta resmi yang tercatat di internet/kontrak broker dari model estimasi terburuk (*worst-case stress test*), serta memvalidasinya terhadap bukti empiris dari aplikasi **MetaTrader 4 Android (Akun Real MRG)**.

---

## 1. Pemetaan Lengkap Variabel Kendali Admin Broker

Di arsitektur server MetaTrader, admin broker memiliki akses ke **MetaTrader Administrator Tool**, **Liquidity Bridge/Gateway**, dan modul **Virtual Dealer Plugin (VDP)**. Berikut adalah seluruh variabel yang dapat diatur oleh admin broker:

| No | Nama Variabel / Parameter | Fungsi & Mekanisme Kerja di MT4/MT5 Server |
| :---: | :--- | :--- |
| **1** | **`MODE_SPREAD` (Spread Markup)** | Menambahkan margin pips tetap/mengambang di atas harga pasar mentah (*raw ECN liquidity*). Admin dapat mengatur markup simetris (Bid & Ask sama) atau asimetris (misal menaikkan Ask lebih tinggi saat pasar bullish). |
| **2** | **`MODE_STOPLEVEL` (Stop Level)** | Jarak minimum (dalam satuan *points*) dari harga pasar terkini di mana pending order (Buy/Sell Limit/Stop) dan Stop Loss / Take Profit diizinkan untuk dipasang. Jika order dipasang lebih dekat dari jarak ini, server melempar error `ERR_INVALID_STOPS` (130). |
| **3** | **`MODE_FREEZELEVEL` (Freeze Level)** | Jarak harga di mana order yang mendekati SL/TP "dibekukan". Trader tidak dapat membatalkan, mengubah harga, atau menutup posisi secara manual saat harga berada dalam rentang freeze level. |
| **4** | **`MODE_SWAPLONG` & `MODE_SWAPSHORT`** | Nilai bunga menginap (*overnight financing fee*) per malam untuk posisi Buy dan Sell. Admin broker retail umumnya menetapkan bunga **kedua arah bernilai negatif** untuk mengambil margin keuntungan dari selisih suku bunga perbankan. |
| **5** | **`MODE_SWAPROLLOVER3DAYS` (Triple Swap)** | Hari di mana beban bunga menginap dikalikan 3x lipat untuk memperhitungkan akhir pekan (Sabtu & Minggu saat pasar tutup). Standar forex adalah hari **Rabu malam ke Kamis dini hari**. |
| **6** | **`MODE_COMMISSION` (Komisi Transaksi)** | Tarif tunai per volume transaksi (misal $/lot, $/deal, atau % per turn). Diterapkan baik saat pembukaan (open) atau pembagian separuh saat open dan separuh saat close. |
| **7** | **Execution Mode (Instant vs Market)** | *Instant Execution* memvalidasi deviasi harga (memicu *Requote* jika harga bergerak), sedangkan *Market Execution* selalu mengeksekusi order pada harga terbaik berikutnya di antrian likuiditas (memicu *Slippage*). |
| **8** | **Virtual Dealer Artificial Delay (VDP)** | Menambahkan jeda waktu eksekusi buatan (misal +200ms hingga +1.500ms) khusus pada akun trader yang di-profiling (misal akun yang menggunakan EA cepat atau sering profit). |
| **9** | **Asymmetric Slippage Capping** | Mengatur batas *positive slippage* menjadi nol (keuntungan lonjakan harga diserap broker), sementara *negative slippage* dibiarkan lolos penuh hingga batas maksimum yang diizinkan. |
| **10** | **Rollover Spread Widening Window** | Penipisan likuiditas harian terjadwal pada pukul 23:58 – 00:05 server time. Admin membiarkan spread melebar 3x s/d 8x lipat untuk mencerminkan pergantian buku kliring antar-bank global (*end-of-day settlement*). |
| **11** | **Dynamic Margin & Leverage Restrictions** | Otomatis memangkas leverage akun (misal dari 1:100 menjadi 1:20) pada 15-30 menit sebelum rilis berita berkategori *High Impact* (NFP, CPI) atau sebelum penutupan pasar akhir pekan (Jumat sore). Hal ini memperbesar margin yang dibutuhkan dan dapat memicu *Margin Call* mendadak. |
| **12** | **Deposit & Withdrawal Exchange Markup** | Selisih kurs internal broker untuk mata uang IDR/USD. Misalnya mematok kurs deposit Rp 15.600/USD tetapi kurs penarikan Rp 15.100/USD (~3.2% potongan tersembunyi atas seluruh laba yang ditarik). |
| **13** | **B-Book Routing & Client Tagging** | Algoritma *A/B-Book profiling*. Aliran order trader yang konsisten rugi ditahan di B-Book (keuntungan internal broker), sedangkan trader yang konsisten menang dilempar ke A-Book (STP) dengan penambahan spread/komisi eksternal. |

---

## 2. Pemilahan: Fakta Resmi di Internet vs Model Estimasi Terburuk Kita

Berikut adalah komparasi tegas antara parameter yang **disebutkan secara eksplisit** di internet/dokumen resmi Traders Family & MRG Mega Berjangka, berbanding dengan **model estimasi terburuk (*worst-case stress test*)** yang kita pasang di sistem backtest/forward test:

| Parameter Biaya | Fakta Resmi di Internet & Regulasi Bappebti (MRG / TF) | Model Estimasi Terburuk Kita (*Worst-Case Stress Test*) | Rationale & Alasan Penerapan |
| :--- | :--- | :--- | :--- |
| **Entitas & Regulasi** | PT MRG Mega Berjangka, teregulasi resmi Bappebti, kliring KBI/ICDX. Bermitra resmi dengan komunitas Traders Family. | Diasumsikan broker retail B-Book dengan friksi maksimal. | Kita menguji strategi agar tetap profit bahkan jika broker memberikan kondisi terburuk. |
| **Komisi Transaksi** | • *Akun Free Commission*: $0 komisi (spread dinaikkan).<br>• *Akun Komisi*: Sekitar **$1 per 0.1 lot** = **$10 per 1.0 lot**. | **`1.0 pip/lot`** ($10/lot pada EURUSD/major pair). | Menghitung beban komisi termahal yang berlaku di akun komisi. |
| **Spread Major Pairs (EURUSD, AUDUSD, GBPUSD)** | Floating spread mulai dari 0.8 pips (kondisi ideal), tipikal floating retail 1.4 – 2.0 pips. | **`2.2 pips`** (22 points). | Mengasumsikan spread selalu berada di titik terlebar pada sesi aktif, bukan titik terendahnya. |
| **Spread Cross Pairs (AUDCAD, AUDJPY, EURGBP)** | Floating spread tipikal 3.5 – 5.0 pips. | **`5.5 pips`** (55 points). | Memberikan ruang penyusutan bagi pasangan mata uang silang (*cross currency*). |
| **Spread Emas (XAUUSD)** | Floating spread tipikal $0.35 – $0.65 (35–65 points). | **`8.0 pips`** ($0.80 / 80 points). | Menahan volatilitas komoditas emas saat pergerakan tajam New York. |
| **Slippage Eksekusi** | Market execution: Normal 0.1 – 0.5 pips. Tidak ada jaminan nol slippage saat volatilitas tinggi. | **`1.0 pip`** (Asymmetric Negative Slippage). | Mengasumsikan setiap order selalu tergelincir 1 pip penuh melawan arah kita. |
| **Biaya Inap (Swap)** | Dikenakan per malam sesuai selisih suku bunga. Hari Rabu dikenakan Triple Swap (3x). | **`-0.7 pips/malam`** (`-7.0 points`) Buy & Sell + **Triple Swap Rabu (`-2.1 pips`)**. | Mengasumsikan bunga menginap selalu negatif di kedua arah (B-Book financing drag). |
| **Stop Level Jarak Order** | Ditentukan oleh broker, tipikal 20 – 30 points (2.0 – 3.0 pips). | **`25 points`** (`2.5 pips`) ditegakkan via `enforce_stop_levels = true`. | Mencegah penolakan order MT4 `ERR_INVALID_STOPS` saat sinyal dieksekusi. |
| **Total Friksi Sekali Masuk & Keluar** | Rata-rata riil: ~1.8 – 2.5 pips. | **`4.2 – 4.5 pips`** (Major) / **`7.5 pips`** (Cross). | **Stress Test**: Jika strategi lolos di beban 4.2+ pips, maka di realita akun MRG hasilnya akan jauh lebih menguntungkan. |

---

## 3. Bukti Empiris dari Aplikasi MT4 Android (Akun Real MRG)

Data kuotasi riil yang diambil langsung dari aplikasi MT4 Android pada tanggal 18 Oktober pukul **22:54:59** (penutupan sesi London / pertengahan sesi New York):

| Ticker MT4 | Harga Bid | Harga Ask | Spread Layar (Points) | Konversi ke Pips Riil | Range Harian (Low - High) | Kategori Tier TF |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`XAUUSD_MRG`** | 4163.96 | 4164.59 | **63** | **6.3 pips** ($0.63) | 4148.10 - 4198.63 | Tier 4 (Komoditas Emas) |
| **`AUDUSD_MRG`** | 0.69439 | 0.69453 | **14** | **1.4 pips** | 0.69135 - 0.69671 | Tier 1 (Major Pair) |
| **`EURUSD_MRG`** | 1.12387 | 1.12401 | **14** | **1.4 pips** | 1.12079 - 1.12693 | Tier 2 (Major Pair) |
| **`GBPUSD_MRG`** | 1.33272 | 1.33287 | **15** | **1.5 pips** | 1.32631 - 1.33715 | Tier 2 (Major Pair) |
| **`USDCHF_MRG`** | 0.82394 | 0.82412 | **18** | **1.8 pips** | 0.82176 - 0.82821 | Tier 1 (Major Pair) |
| **`USDJPY_MRG`** | 157.980 | 157.999 | **19** | **1.9 pips** | 157.691 - 158.337 | Tier 3 (JPY Major) |
| **`USDCAD_MRG`** | 1.41373 | 1.41393 | **20** | **2.0 pips** | 1.41341 - 1.41987 | Tier 2 (Major Pair) |
| **`AUDCAD_MRG`** | 0.98162 | 0.98208 | **46** | **4.6 pips** | 0.98123 - 0.98613 | Tier 2 (Cross Pair) |
| **`AUDCHF_MRG`** | 0.57211 | 0.57249 | **38** | **3.8 pips** | 0.57133 - 0.57457 | Tier 1 (Cross Pair) |
| **`AUDJPY_MRG`** | 109.689 | 109.740 | **51** | **5.1 pips** | 109.233 - 110.057 | Tier 2 (Cross Pair) |

> [!NOTE]
> **Catatan Pembacaan Satuan**:
> Pada platform MT4 dengan 5-digit kuotasi (3-digit untuk pair JPY), angka "Spread" yang ditampilkan di layar adalah dalam satuan **POINTS (0.1 pip)**, bukan pips penuh:
> * Spread `14` = **1.4 pips**
> * Spread `15` = **1.5 pips**
> * Spread `20` = **2.0 pips**
> * Spread `46` = **4.6 pips**
> * Spread `51` = **5.1 pips**
> * Spread `63` pada Emas = **6.3 pips** ($0.63 / 63 sen)

---

## 4. Evaluasi Pertanyaan: "Pernah Lihat Spread Sampai 60, Apakah Sesuai Ekspektasi?"

### A. Ya, Sangat Sesuai Ekspektasi!
Mengapa Anda pernah melihat angka spread mencapai **60** di layar MT4?
1. **Untuk Pasangan Mata Uang Silang (Cross Pairs)**:
   - Seperti yang tertera pada snapshot di atas, `AUDJPY_MRG` dalam kondisi normal saja sudah berada di angka **51 points (5.1 pips)**, dan `AUDCAD_MRG` di **46 points (4.6 pips)**.
   - Saat ada sedikit gejolak harga atau likuiditas bergerak, angka spread melonjak dari 51 menjadi 60 points (6.0 pips) adalah **hal yang sangat wajar dan rutin terjadi pada cross pair di broker retail**.
2. **Untuk Pasangan Mata Uang Utama (Major Pairs seperti EURUSD/GBPUSD)**:
   - Normalnya berada di angka 14 – 15 points (1.4 – 1.5 pips).
   - Namun angka ini **bisa melonjak ke 60 points (6.0 pips)** pada 3 momen khusus:
     - **Pukul 23:59 – 00:05 Server Time (Rollover Window)**: Saat pasar New York tutup dan likuiditas global tipis sebelum sesi Tokyo/Sydney aktif.
     - **Rilis Berita Berdampak Tinggi (High-Impact News)**: Beberapa detik saat NFP, US CPI, atau pengumuman suku bunga bank sentral dirilis.
     - **Pembukaan Pasar Hari Senin Dini Hari**: Saat *gap* akhir pekan dibuka.

### B. Penyesuaian Arsitektur Kuantitatif Kita
Data empiris MT4 ini membuktikan bahwa:
* Model terburuk kita untuk **Major Pair** (`simulation_spread_pips: 2.2 pips` = 22 points) sudah sangat konservatif dan mencakup 90%+ kondisi normal (1.4–1.5 pips).
* **Penyesuaian Diperlukan**: Untuk **Cross Pair** dan **Emas**, mesin backtest tidak boleh menggunakan spread rata 2.2 pips, melainkan harus menggunakan **Tier-Aware Spread**:
  - **Tier 1 & 2 Major (EURUSD, AUDUSD, GBPUSD)**: Model Terburuk = `2.2 pips` (22 points)
  - **Tier 1 & 2 Minor/USD (USDCHF, USDJPY, USDCAD)**: Model Terburuk = `2.5 pips` (25 points)
  - **Tier 2 & 3 Cross Pairs (AUDCAD, AUDCHF, AUDJPY, EURGBP, GBPJPY)**: Model Terburuk = `5.5 pips` (55 points, mengantisipasi lonjakan hingga 60 points)
  - **Tier 4 Komoditas Emas (XAUUSD)**: Model Terburuk = `8.0 pips` (80 points / $0.80)
