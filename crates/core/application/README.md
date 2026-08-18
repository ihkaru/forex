# 📦 Crate: `application` (Hexagonal Use Cases)

Direktori ini berisi lapisan aplikasi yang mengorkestrasi interaksi antara **Domain**, **Ports**, dan berbagai strategi trading.

## Layanan Utama (Services)
- `SignalEngineService`: Orkestrator pipeline penghasil sinyal otomatis (scanning market data, evaluasi rule strategi, penyimpanan ke database, dan broadcast ke subscriber Trader Family/Telegram).
- `BacktestService`: Mesin penguji strategi secara historis (deterministik).

## Aturan Arsitektur
- Bergantung hanya pada crate `domain`.
- Menerima dependensi adapter secara polimorfik melalui `Arc<dyn Port>` atau static generics.
