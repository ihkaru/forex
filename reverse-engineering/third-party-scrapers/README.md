# 📦 Directory: `reverse-engineering/third-party-scrapers`

Tempat menaruh clone repositori publik (Python, TypeScript, Node.js, Go) dari GitHub sebagai referensi logika parsing dan struktur endpoint sebelum di-rewrite ke Rust.

## Panduan Pemanfaatan Folder
1. **Clone Repo Referensi**:
   ```bash
   git clone https://github.com/someuser/python-forexfactory-scraper reverse-engineering/third-party-scrapers/python-forexfactory
   ```
2. **Analisis Algoritma**:
   - Identifikasi selector HTML (`css selector` / `xpath`), regex pattern, atau header request.
3. **Rewrite ke Rust**:
   - Implementasikan logic yang sama di `crates/scrapers/<nama-scraper>-rs/` menggunakan crate `reqwest` dan `scraper`.
4. **Hapus / Ignore**:
   - Pastikan repo referensi tidak di-commit ke Git jika ukurannya besar (sudah ter-cover oleh `.gitignore`).
