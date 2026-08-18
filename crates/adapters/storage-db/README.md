# 🗄️ Crate: `storage-db` (Tiered Hybrid Storage Adapter)

Adapter persistensi data bertingkat (*Tiered Storage*) yang mengimplementasikan `domain::ports::StoragePort` untuk mendukung kebutuhan riset kuantitatif dan sistem produksi 24/7.

---

## 🏛️ Arsitektur Tiered Storage

1. **`InMemoryStorage` (Hot State / Unit Testing)**:
   - Penyimpanan thread-safe berbasis RAM murni (`Arc<RwLock<...>>`).
   - Digunakan untuk *unit test*, *mock double*, dan *sub-microsecond live tick buffering*.

2. **`SqlxStorage` (TimescaleDB / PostgreSQL / SQLite)**:
   - Driver persistensi transaksional ACID berbasis `sqlx`.
   - Menggunakan skema migrasi [migrations/001_initial_schema.sql](file:///home/ihza/Projects/forex/crates/adapters/storage-db/migrations/001_initial_schema.sql).
   - Mendukung **TimescaleDB Hypertables** untuk pemartisian otomatis candlestick per rentang waktu.
   - Menyimpan riwayat sinyal, order eksekusi, dan log publikasi **Trader Family**.

3. **`DuckDbAnalyticalEngine` (In-Process OLAP & Parquet Backtesting)**:
   - Mesin analitik kolumnar berkecepatan tinggi untuk membaca dan mengekstrak data historis multi-tahun dari file kompresi `.parquet`.

---

## 🚀 Migrasi Skema Database

Skema SQL otomatis dijalankan saat container TimescaleDB menyala via `docker-compose.yml` atau secara programatik melalui `sqlx::migrate!`.
