# 📑 Dokumentasi Endpoint API Trader Family (Reverse Engineered)

Dokumentasi spesifikasi API internal hasil tangkapan mitmproxy dan dekompilasi APK.

---

## 1. Authentication
- **Endpoint**: `POST /api/v1/auth/login`
- **Headers**:
  ```http
  Content-Type: application/json
  User-Agent: TraderFamily-Android/2.4
  ```
- **Request Body**:
  ```json
  {
    "email": "user@example.com",
    "password": "hashed_or_plain_password"
  }
  ```
- **Response**:
  ```json
  {
    "access_token": "eyJhbGciOi...",
    "refresh_token": "d7a8...",
    "user_id": "usr_12345"
  }
  ```

---

## 2. Publish Signal to Channel
- **Endpoint**: `POST /api/v1/channels/{channel_id}/signals`
- **Headers**:
  ```http
  Authorization: Bearer <access_token>
  Content-Type: application/json
  ```
- **Request Body**:
  ```json
  {
    "symbol": "EURUSD",
    "action": "BUY",
    "entry_price": 1.08500,
    "stop_loss": 1.08300,
    "take_profit_1": 1.08900,
    "take_profit_2": 1.09200,
    "description": "Asian range liquidity sweep confirmed on M15."
  }
  ```

---

## 3. Update Signal Status
- **Endpoint**: `PATCH /api/v1/channels/{channel_id}/signals/{signal_id}`
- **Request Body**:
  ```json
  {
    "status": "TARGET_1_HIT",
    "closing_price": 1.08900
  }
  ```
