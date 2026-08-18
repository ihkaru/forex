# 📅 Crate: `forexfactory-rs` (Economic Calendar Scraper)

Scraper Rust independen untuk mengekstraksi jadwal kalender ekonomi ForexFactory secara real-time.
- Mengimplementasikan `domain::ports::EconomicCalendarPort`.
- Digunakan oleh signal engine untuk mem-filter dan menghindari open position saat *High Impact News* (NFP, FOMC, CPI).
