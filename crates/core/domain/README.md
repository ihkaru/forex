# 📦 Crate: `domain` (Hexagonal Core)

Direktori ini berisi inti domain logic (Core Domain) yang **murni (Pure Rust)** tanpa I/O, database, ataupun dependensi jaringan.

## Struktur
- `models/`: Entitas & Value Objects (`Symbol`, `Candle`, `Tick`, `Signal`, `Order`, `RiskProfile`).
  - Menggunakan `rust_decimal::Decimal` untuk mencegah presisi error floating-point pada kalkulasi harga Forex.
- `ports/`: Trait Interfaces (Interface-First Pattern).
  - `MarketDataPort`: Kontrak penarikan tick dan candle.
  - `SignalPublisherPort`: Kontrak distribusi sinyal (Trader Family, Telegram, dsb).
  - `StoragePort`: Kontrak penyimpanan data.
  - `StrategyPort`: Kontrak evaluasi strategi.
  - `EconomicCalendarPort` & `SentimentPort`: Kontrak data scraper.
- `errors.rs`: Domain error enum (`DomainError`).

## Aturan Pengembangan
1. **Dilarang keras melakukan I/O langsung** (Network HTTP, Disk IO, DB query) di dalam crate ini.
2. Semua I/O harus didefinisikan sebagai trait di dalam folder `ports/`.
