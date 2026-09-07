# 📑 Spesifikasi REST API Traders Family (Verified Live 2026)

Dokumentasi ini adalah hasil reverse engineering langsung dari **POCO F7 (Android HyperOS)** dan analisis biner **Flutter AOT (`libapp.so`)** aplikasi resmi Traders Family (`com.tradersfamily.app`).

---

## 🌐 Base URL & Microservice Gateways

Seluruh API berjalan di atas Cloudflare Edge dengan host gateway:
```http
Base Host: https://app3.tradersfamily.app
```

Terdapat 3 Microservices Gateway utama:
1. `/traders/api/v1/` : Manajemen Akun, Autentikasi, Profil Analis, Notifikasi.
2. `/ois/api/v1/`     : Order Information System (Channel, Sinyal, Copytrade, Scoring 7-Pilar, Valued Pips).
3. `/mrg/api/v1/`     : Integrasi Broker Partner (MRG Mega Berjangka / Askap Social Trade, MT4/MT5 Demo & Real).

---

## 🔐 1. Authentication & Session Lifecycle

### A. Login dengan Email & Password
- **Method & URL**: `POST https://app3.tradersfamily.app/traders/api/v1/login/`
- **Headers**:
  ```http
  Content-Type: application/json
  User-Agent: Dart/3.4 (dart:io)
  ```
- **Request Body**:
  ```json
  {
    "email": "analyst@example.com",
    "password": "PasswordAnda123"
  }
  ```
- **Response Success (HTTP 200)**:
  ```json
  {
    "status": "success",
    "data": {
      "accessToken": "eyJhbGciOi...",
      "refreshToken": "d7a8b9c0...",
      "expiresIn": 86400,
      "user": {
        "id": "usr_xxxxxxxx",
        "email": "analyst@example.com",
        "name": "Quant Trader"
      }
    }
  }
  ```
- **Error Responses**:
  - `{"message":"REQUEST_PARAMETER_MUST_CONTAIN_EMAIL_AND_PASSWORD","error":"MissingRequestError"}`
  - `{"message":"LOGIN_FAILED_PLEASE_CHECK_EMAIL_OR_USERNAME","error":"UnknownUserError"}`

### B. Auto-Refresh Token
- **Method & URL**: `POST https://app3.tradersfamily.app/traders/api/v1/refreshlogin/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  Content-Type: application/json
  ```
- **Request Body**:
  ```json
  {
    "refreshToken": "d7a8b9c0..."
  }
  ```
- **Response Success (HTTP 200)**:
  ```json
  {
    "status": "success",
    "data": {
      "accessToken": "new_eyJhbGciOi...",
      "refreshToken": "new_d7a8b9c0..."
    }
  }
  ```

---

## 📡 2. Channel & Scoring Overview

### A. Dapatkan Informasi Channel Saya
- **Method & URL**: `GET https://app3.tradersfamily.app/ois/api/v1/my-channel/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  ```
- **Response**: Mengembalikan detail `channel_id`, jumlah subscriber, level/medal (`Master`/`Legend`), dan status channel.

### B. Monitoring 7-Pilar Scoring
- **Method & URL**: `GET https://app3.tradersfamily.app/ois/api/v1/channel/scoring/overview/`
- **Method & URL (Histori)**: `GET https://app3.tradersfamily.app/ois/api/v1/channel/scoring/history/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  ```

### C. Estimasi Valued Pips (VP)
- **Method & URL**: `POST https://app3.tradersfamily.app/ois/api/signal/vp/estimation/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  Content-Type: application/json
  ```

---

## 🎯 3. Pembuatan & Pengelolaan Sinyal (Hilir Engine)

### A. Validasi Sinyal (Pre-Submission Check)
- **Method & URL**: `GET https://app3.tradersfamily.app/traders/api/v1/check/signal/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  ```

### B. Publikasi Sinyal Baru (Create Signal)
- **Method & URL**: `POST https://app3.tradersfamily.app/ois/api/v1/signal/create/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  Content-Type: application/json
  ```
- **Request Body**:
  ```json
  {
    "channel_id": "tf_channel_id_anda",
    "symbol": "EURUSD",
    "op": 0,
    "open_price": "1.08500",
    "sl": 1.08000,
    "tp": 1.09000,
    "lot": 0.01,
    "expired": 24,
    "description": "Pola N Institutional Golden Pocket 61.8% Retracement"
  }
  ```
  *Keterangan Nilai `op`*:
  - `0`: Buy Limit
  - `1`: Sell Limit
  - `2`: Buy Stop
  - `3`: Sell Stop

### C. Proteksi & Validasi SL/TP
- **Method & URL**: `POST https://app3.tradersfamily.app/ois/api/v1/signal/create/protection/`
- **Fungsi**: Memverifikasi rasio Risk:Reward (1:1.0 s.d. 1:3.0) dan jarak SL terhadap level tier pair.

### D. Update Sinyal (SL/TP Touch / Running)
- **Method & URL**: `POST https://app3.tradersfamily.app/ois/api/v1/signal/update/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  Content-Type: application/json
  ```

### E. Batalkan Sinyal Pending (Cancel Order)
- **Method & URL**: `POST https://app3.tradersfamily.app/ois/api/v1/signal/cancel/`
- **Headers**:
  ```http
  Authorization: Bearer <accessToken>
  Content-Type: application/json
  ```
- **Request Body**:
  ```json
  {
    "signal_id": "sig_xxxxxxxx"
  }
  ```

---

## ⚡ 4. Ringkasan Integrasi ke Rust Engine

Semua endpoint di atas kompatibel langsung dengan crate adapter [`crates/adapters/publisher-traderfamily`](file:///home/ihza/Projects/forex/crates/adapters/publisher-traderfamily):

```rust
// Inisialisasi Publisher dengan Gateway app3
let tf_config = TraderFamilyConfig {
    base_url: "https://app3.tradersfamily.app".to_string(),
    auth_token: env::var("TF_AUTH_TOKEN").unwrap_or_default(),
    channel_id: "your_channel_id".to_string(),
    user_agent: "Dart/3.4 (dart:io)".to_string(),
};
```
*Tidak ada dependensi ke GUI, emulator Android, ataupun browser wrapper.*
