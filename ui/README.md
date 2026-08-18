# 🖥️ Forex Quant Dashboard: Architecture & Frontend Standard

Frontend dashboard untuk monitoring sinyal kuantitatif Forex dan auto-posting ke **Trader Family Channel** mengadopsi **Feature-Sliced Design (FSD)** yang dipadukan dengan **Hexagonal Architecture (Ports & Adapters)** berbasis **TypeScript & Svelte 5 / Vanilla Ultra-Fast DOM**.

---

## 🏛️ Arsitektur Lapisan (Feature-Sliced Design)

```
src/
├── app/                  # App Entry, Global Providers & Layout Setup
├── pages/                # Halaman Dashboard, Backtest Lab, Strategy Config
├── widgets/              # Komposisi UI Terpadu (SignalFeed, PhonePreview, MetricsGrid)
├── features/             # Interaksi Bisnis (ScanMarketAction, BroadcastSignalAction)
├── entities/             # [HEXAGON DOMAIN] Pure TS Models (Signal, Candle, Subscriber)
└── shared/               # [HEXAGON PORTS & ADAPTERS]
    ├── ports/            # TypeScript Interfaces (MarketStreamPort, SignalRepositoryPort, TraderFamilyPort)
    ├── adapters/         # Implementasi Konkret (WebSocketMarketAdapter, RestSignalAdapter)
    └── ui/               # Reusable Glassmorphic UI Kit (Button, Card, Badge, Modal)
```

---

## 🛡️ Penegakan Batas Arsitektur (ESLint Boundaries)

Arsitektur ditegakkan secara ketat melalui `.eslintrc.cjs` dengan plugin `eslint-plugin-boundaries`.
Arah panah ketergantungan wajib **Inward-Only**:

- `shared/` ➔ **Dilarang** meng-import `entities`, `features`, `widgets`, atau `pages`.
- `entities/` ➔ Hanya boleh meng-import `shared`.
- `features/` ➔ Hanya boleh meng-import `entities` dan `shared`.
- `widgets/` ➔ Hanya boleh meng-import `features`, `entities`, dan `shared`.
- `pages/` ➔ Boleh meng-import seluruh lapisan di bawahnya.

Jika terjadi impor ilegal (misal: layer `entities` mencoba meng-import UI komponen atau framework hook), **ESLint & CI/CD akan menggagalkan build secara instan**.

---

## 🚀 Menjalankan Frontend

```bash
# Jalankan dalam mode development
npm run dev

# Jalankan linter audit batas arsitektur
npm run lint:architecture
```
