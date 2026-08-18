# 🔌 Crate: `adapters` (Driven / Infrastructure Adapters)

Direktori ini menampung implementasi konkret dari **Outbound Ports** yang didefinisikan pada `domain::ports`.

Setiap adapter di sini dapat dijadikan **Git Submodule** independen jika ingin dipisahkan ke repositori GitHub masing-masing.

## Daftar Adapter
1. `publisher-traderfamily/`: Klien API Trader Family hasil reverse engineering untuk auto-post sinyal ke channel VIP subscriber.
2. `notifier-telegram/`: Adapter pengirim sinyal & notifikasi darurat via Telegram Bot API.
3. `storage-db/`: Adapter database (TimescaleDB / SQLite / DuckDB / In-Memory).
4. `broker-connector/`: Bridge komunikasi ke broker / platform trading (MetaTrader 5 ZeroMQ, cTrader Open API).
