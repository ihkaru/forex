# 🔍 Directory: `reverse-engineering` (Scraper & Protocol Reverse Engineering)

Direktori ini didedikasikan untuk riset dekompilasi, *network packet capture*, dan inspeksi protokol dari sumber data eksternal sebelum di-rewrite ke Rust.

## Struktur Sub-Direktori
- `trader-family/`: Investigasi aplikasi Android **Trader Family** (analisis APK, script Frida bypass SSL Pinning, dokumentasi endpoint REST & WebSocket).
- `third-party-scrapers/`: Tempat menyimpan repositori referensi publik (Python/Node.js/Go) dari GitHub sebelum dibuatkan implementasi murni Rust di `crates/scrapers/`.

## Workflow Rewrite ke Rust
1. **Analisis Protokol**: Tangkap request/response menggunakan `mitmproxy` atau `Charles`.
2. **Reverse APK/Source**: Bongkar struktur data, parameter signing/HMAC, atau format payload JSON.
3. **Dokumentasikan**: Catat URL endpoint, method, headers, dan struktur response di `docs/api_endpoints.md`.
4. **Rewrite ke Rust**: Buat crate baru di `crates/adapters/` atau `crates/scrapers/` yang mengimplementasikan trait dari `domain::ports`.
