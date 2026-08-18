# 🚀 Directory: `apps` (Executable Daemons & Entrypoints)

Direktori ini berisi executable binary crates (entrypoints) yang menggabungkan (*composition root* / Dependency Injection) seluruh crate `core`, `adapters`, dan `scrapers`.

## Daftar Aplikasi
1. `signal-daemon/`: Autonomous daemon utama yang berjalan 24/7 di server untuk membaca data pasar, mendeteksi sinyal, dan memposting ke Trader Family Channel.
2. `api-server/`: REST & WebSocket backend API (Axum) untuk melayani Web UI Dashboard.
3. `scraper-worker/`: Background worker yang secara terjadwal menarik dan menyimpan data makro ekonomi, sentimen, dan histori harga.
4. `cli/`: Tool antarmuka baris perintah untuk pengujian manual sinyal, eksekusi backtest, dan manajemen konfigurasi.
