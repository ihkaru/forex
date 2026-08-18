# 🇨🇭 Crate: `dukascopy-rs` (Dukascopy Bank True-Tick Scraper)

Library penarikan data tick Forex dan Komoditas murni dari **Dukascopy Bank SA (Swiss)** secara gratis tanpa batasan API Key untuk keperluan riset dan backtesting kuantitatif multi-tahun.

---

## 🚀 Fitur Utama
1. **URL Construction Otomatis**: Mendukung konversi pair (e.g. `EUR/USD` ➔ `EURUSD`) dan perhitungan 0-indexed month.
2. **Decompressor Biner `.bi5`**: Membongkar file biner 20-byte terkompresi LZMA berkecepatan tinggi.
3. **Tick Aggregator**: Mengonversi jutaan raw tick milidetik menjadi Candlestick time-series (M1, M5, M15, H1, H4, D1) dengan presisi `rust_decimal::Decimal`.
4. **Zero-API Key**: Berjalan langsung via HTTP public data feed Dukascopy.
