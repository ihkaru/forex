# 🚀 Panduan Lengkap Deployment ke Coolify (Ubuntu Desktop 24 Jam)

Dokumen ini adalah panduan langkah demi langkah untuk men-deploy sistem kuantitatif trading **Forex Pola N** ke instance **Coolify** yang berjalan di **Ubuntu Desktop 24 Jam** Anda.

---

## 🏗️ 1. Arsitektur Komponen

```
                  SERVER UBUNTU DESKTOP ANDA (HIDUP 24 JAM)
 ┌────────────────────────────────────────────────────────────────────────┐
 │                                                                        │
 │  1. APLIKASI NATIVE (Wine GUI)                                         │
 │     [ MetaTrader 4 Akun Anda Sendiri ]                                 │
 │              │                                                         │
 │              │  EA Bridge (ForexHexagonBridge) kirim tick              │
 │              ▼  ke TCP socket localhost:5555                           │
 │                                                                        │
 │  2. CONTAINER DOCKER (Dikelola Otomatis oleh Coolify)                  │
 │     ┌────────────────────────────────────────────────────────────┐     │
 │     │  `api-server` (Rust)                                       │     │
 │     │  • Bind port 127.0.0.1:5555 (Menerima feed MT4 aman)       │     │
 │     │  • Engine Pola N & Publikasi ke Cloud Traders Family       │     │
 │     └────────────────────────────┬───────────────────────────────┘     │
 │                                  │ (Internal Network: Port 5000)       │
 │     ┌────────────────────────────▼───────────────────────────────┐     │
 │     │  `ui` (Svelte 5 + Nginx Reverse Proxy)                     │     │
 │     │  • Melayani dashboard TradingView Canvas                   │     │
 │     │  • Mem-proxy /api & WebSocket ke backend tanpa CORS        │     │
 │     └────────────────────────────┬───────────────────────────────┘     │
 └──────────────────────────────────┼─────────────────────────────────────┘
                                    │
                         Port 3000 / HTTPS 443
                                    │
                                    ▼
       🌐 Domain Publik Anda (e.g. https://forex.domainanda.com)
          (Dapat diakses dari Laptop, HP, & Tablet dari mana saja)
```

---

## ⚡ 2. Mengapa Menggunakan Custom Dockerfile (`cargo-chef`)?

Sesuai riset standar arsitektur Rust per September 2026:
- **Bukan Nixpacks**: Nixpacks sering gagal mendeteksi dependensi monorepo dan mengompilasi ulang ribuan *crates* dari nol (butuh 10–15 menit tiap push).
- **`cargo-chef`**: Seluruh dependensi pihak ketiga di-cache dalam layer Docker khusus.
  - Push pertama: ~3–5 menit (men-download & mengompilasi seluruh dependensi).
  - Push berikutnya: **< 30 detik!** (Hanya kode logika aplikasi Anda yang dikompilasi).

---

## 🛠️ 3. Langkah Setup di Dashboard Coolify

1. **Buka Dashboard Coolify** di browser Anda.
2. Klik **Projects** $\rightarrow$ Pilih Environment Anda (misal: `Production`).
3. Klik **+ Add New Resource** $\rightarrow$ Pilih **Docker Compose**.
4. Pilih sumber: **GitHub App** $\rightarrow$ Pilih repository Anda: `forex`.
5. Pada bagian **Compose File**, pilih opsi:
   - **Docker Compose Location**: `deploy/docker-compose.coolify.yml`
6. Buka tab **Domains / Configuration**:
   - Pada service **`ui`**, masukkan domain:
     ```text
     https://forex.dvlpid.my.id
     ```
   - **Cloudflared & Traefik Notes**:
     - Karena domain Anda menggunakan wildcard `*.dvlpid.my.id` yang diarahkan ke Traefik via Cloudflare Tunnel (`cloudflared`), Traefik Coolify akan secara otomatis mencocokkan routing Host rule `Host(`forex.dvlpid.my.id`)` ke service `ui` container.
     - Di Cloudflare Dashboard (tab Network), pastikan fitur **WebSockets** aktif (default: ON) agar data tick MT4 dapat mengalir lancar.
7. Buka tab **Environment Variables** (Opsional untuk auto-login Traders Family):
   ```env
   TF_EMAIL=email_analis_anda@gmail.com
   TF_PASSWORD=password_tf_anda
   TF_CHANNEL_ID=nomor_channel_anda
   RUST_LOG=info
   ```
8. Klik **Deploy**! 🚀
   - Coolify akan menjalankan build multi-stage Rust (`cargo-chef`) dan build frontend Svelte (`npm run build`).

---

## 📈 4. Langkah Menghubungkan MetaTrader 4 di Ubuntu Desktop

Karena server Anda adalah Ubuntu Desktop, MT4 akun broker Anda sendiri dapat berjalan langsung di desktop host via Wine:

1. **Pastikan MT4 Terinstall & Login**:
   - Jalankan MT4 broker pilihan Anda di Ubuntu Desktop:
     ```bash
     wine ~/.wine/drive_c/.../terminal.exe
     ```
   - Login ke akun trading Anda (broker apa pun bebas: Exness, IC Markets, Vantage, MRG, dll.).
2. **Pasang EA Bridge**:
   - Salin berkas EA dari repo:
     [`crates/adapters/broker-connector/mql4/ForexHexagonBridge.mq4`](file:///home/ihza/Projects/forex/crates/adapters/broker-connector/mql4/ForexHexagonBridge.mq4)
   - Buka MT4 $\rightarrow$ Menu **File** $\rightarrow$ **Open Data Folder** $\rightarrow$ Buka `MQL4/Experts/` $\rightarrow$ Paste file tersebut di sana.
   - Buka MetaEditor di MT4, buka file tersebut, dan klik **Compile** (menghasilkan `ForexHexagonBridge.ex4`).
3. **Attach EA ke Chart**:
   - Buka chart instrumen utama Anda (misalnya `XAUUSD` H1 atau `EURUSD` H1).
   - Drag `ForexHexagonBridge` dari panel Navigator ke chart.
   - Di tab **Common**: Centang **"Allow DLL imports"** dan **"Allow live trading"**.
   - Di tab **Inputs**:
     - `InpHost` = `127.0.0.1`
     - `InpPort` = `5555`
   - Klik **OK**.
4. **Verifikasi Koneksi**:
   - Di pojok kanan atas chart MT4 akan muncul senyuman (smiling face).
   - Di tab **Experts** di terminal MT4 akan tertulis:
     `Connected to 127.0.0.1:5555 successfully`.
   - Data tick dan candle berjalan dari akun Anda sekarang mengalir 24/7 ke Coolify!

---

## 🎯 5. Pengalaman Penggunaan Sehari-Hari

1. **Laptop Utama Anda**:
   - Bebas dimatikan, dimasukkan ke tas, atau dibawa bepergian kapan saja tanpa rasa cemas.
2. **Akses Dari Mana Saja**:
   - Buka `https://forex.domainanda.com` dari laptop, tablet, atau smartphone.
   - Chart bergerak real-time dari feed akun MT4 Anda.
   - Jika algoritma Pola N mendeteksi setup golden pocket yang lolos validasi `TfComplianceGuard`, Anda cukup menekan tombol **[ 🚀 Kirim ke Traders Family ]** langsung dari browser Anda!
