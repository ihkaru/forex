# 🌐 Panduan Live Streaming MetaTrader 4 / 5 via Tailscale Mesh Network

Dokumen ini menjelaskan arsitektur dan panduan operasional integrasi live stream data pasar dari **MetaTrader 4 (MT4)** atau **MetaTrader 5 (MT5)** yang berjalan di **Windows PC** menuju **Hexagon Quantitative Engine** di server Linux ThinkStation melalui jaringan **Tailscale Mesh VPN**.

---

## 🏗️ 1. Arsitektur Jaringan & Aliran Data

```
┌────────────────────────────────────────────────────────┐
│                   Windows PC (Client)                  │
│                                                        │
│   MetaTrader 4 / 5 (Akun Demo / Real Broker)           │
│   └── ForexHexagonBridge.mq4 / .mq5                    │
│       ├── Membaca Watchlist Market Watch               │
│       ├── Mengirim Bar H1 Histori                      │
│       └── Stream Per-Tick Real-time (JSON via WinSock) │
└───────────────────────────┬────────────────────────────┘
                            │
                            │ Raw TCP Stream (Port 5555)
                            │ Terenkripsi WireGuard Mesh
                            ▼
┌────────────────────────────────────────────────────────┐
│             Tailscale Mesh VPN (Overlay Network)       │
│             Virtual IP: 100.101.49.42:5555             │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│           ThinkStation Server (Coolify Docker)         │
│                                                        │
│   api-server Container (Port 5555:5555)                │
│   └── BrokerConnector (TCP Socket Listener)            │
│       ├── Idempotent Bar Upsert (Ring Buffer Memory)   │
│       ├── Pola N Strategy Formation Engine             │
│       └── TfComplianceGuard Validator                  │
│                                                        │
│   ui Container (Port 3000 -> 80)                       │
│   └── Dashboard TradingView Svelte 5                   │
│       (https://forex.dvlpid.my.id/)                    │
└────────────────────────────────────────────────────────┘
```

---

## 🌟 2. Mengapa Menggunakan Tailscale?

Pemanfaatan **Tailscale** memberikan sejumlah keunggulan kritis untuk operasional kuantitatif:

1. **Zero Configuration NAT Traversal & Port Forwarding:**
   PC Windows dapat berada di jaringan rumah, kantor, atau hotspot seluler tanpa perlu mengatur port forwarding di router internet.
2. **End-to-End Encryption (WireGuard):**
   Seluruh lalu lintas tick, order book, dan informasi spread dienkripsi penuh antara Windows PC dan server ThinkStation.
3. **Static Private IP (`100.101.49.42`):**
   IP Tailscale server bersifat tetap (*persistent*), sehingga konfigurasi EA tidak akan pernah terputus meskipun IP publik server atau ISP berubah-ubah.
4. **Multi-Endpoint / Multi-PC Ready:**
   Bisa menghubungkan beberapa PC Windows sekaligus (misalnya 1 PC untuk akun demo MRG, 1 PC untuk akun real, dan 1 laptop cadangan).

---

## 🛠️ 3. Langkah Konfigurasi di Windows PC

### Langkah 1: Aktivasi Tailscale
1. Unduh dan pasang aplikasi resmi **Tailscale** untuk Windows dari [tailscale.com](https://tailscale.com/).
2. Login menggunakan akun yang terhubung ke tailnet ThinkStation.
3. Uji koneksi dari PowerShell:
   ```powershell
   ping 100.101.49.42
   ```
   *Pastikan respons `Reply from 100.101.49.42: bytes=32 time=~50ms`.*

---

### Langkah 2: Pemasangan Expert Advisor (EA)
1. Salin file EA dari direktori repositori:
   * **MetaTrader 4:** `crates/adapters/broker-connector/mql4/ForexHexagonBridge.mq4`
   * **MetaTrader 5:** `crates/adapters/broker-connector/mql5/ForexHexagonBridge.mq5`
2. Di MetaTrader Windows, buka menu **File** $\to$ **Open Data Folder** $\to$ masuk ke `MQL4/Experts/` (atau `MQL5/Experts/`).
3. Paste file ke dalam folder tersebut, lalu buka MetaEditor dan tekan **F7 (Compile)**.

---

### Langkah 3: Pengaturan Izin DLL
Karena EA berkomunikasi langsung melalui library Windows Socket (`ws2_32.dll`):
1. Buka MetaTrader $\to$ **Tools** $\to$ **Options** (`Ctrl + O`) $\to$ tab **Expert Advisors**.
2. Centang:
   * `[x] Allow automated trading`
   * `[x] Allow DLL imports`
3. Klik **OK**.

---

### Langkah 4: Konfigurasi Parameter EA
1. Buka **1 chart saja** (misal `XAUUSD`, timeframe `H1`).
2. Pasang EA `ForexHexagonBridge` ke chart.
3. Di tab **Inputs**, masukkan parameter:

| Parameter | Nilai | Penjelasan |
| :--- | :--- | :--- |
| **`InpHost`** | `100.101.49.42` | IP Tailscale server ThinkStation |
| **`InpPort`** | `5555` | Port TCP Socket bridge server |
| **`InpSyncAllWatchlist`** | `true` | **Wajib `true`**: Otomatis menyinkronkan seluruh pair di Market Watch |
| **`InpHistoryLimit`** | `500` | Jumlah bar H1 historis yang dikirim saat pertama connect |
| **`InpTimerSeconds`** | `2` | Interval update heartbeat & bar sinkronisasi |

4. Di tab **Common**, centang **Allow live trading** dan **Allow DLL imports**, lalu klik **OK**.

---

## 🔍 4. Verifikasi dan Monitoring

### A. Di Tab Experts / Journal MetaTrader Windows
Ketika terhubung, tab **Experts** akan menampilkan log sukses:
```text
🚀 Menginisialisasi ForexHexagonBridge ke Rust Quant Server (Host: 100.101.49.42, Port: 5555)...
✅ Sukses terhubung ke Rust Server di 100.101.49.42:5555
📋 Menyinkronkan seluruh simbol dari Market Watch ke Rust Quant Engine...
📦 Mengirim 500 bar histori H1 XAUUSD ke Rust Quant Engine...
✅ Sukses sinkronisasi seluruh pair Market Watch ke Rust Engine.
```

### B. Di Server ThinkStation
Pantau stream data yang masuk secara real-time via tool `coolify-control`:
```bash
coolify-control logs api-server-fmm1i1dj48cq6aq3c3fhiexe-002828326984 --tail 10
```
Log server akan mengonfirmasi:
```text
INFO broker_connector: 🔗 Koneksi baru dari MetaTrader EA: 100.101.x.x:xxxxx
INFO broker_connector: 📡 Broker TCP Socket menerima data streaming...
```

### C. Di Dashboard Web (`https://forex.dvlpid.my.id/`)
1. Buka `https://forex.dvlpid.my.id/`.
2. Pada toolbar dropdown data source, pilih **MRG DEMO** atau **MRG REAL**.
3. Canvas grafik TradingView dan panel analisis kuantitatif akan otomatis menyajikan harga live dan candle hasil stream dari MetaTrader Windows PC Anda.
