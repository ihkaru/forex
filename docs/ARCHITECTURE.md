# Arsitektur & Prinsip Desain — Forex Hexagonal System

> File ini adalah dokumentasi detail arsitektur. Diload oleh agen **on-demand** (JIT),
> bukan selalu aktif. GEMINI.md hanya menyimpan pointer ke sini.

---

## 1. Hexagonal Architecture (Ports & Adapters)

Lapisan sistem dari dalam ke luar:

```
┌─────────────────────────────────────────────────┐
│  crates/core/domain/          ← ZERO I/O        │
│  ├── entities/                                  │
│  ├── value_objects/                             │
│  ├── strategies/  (PolaNStrategy, etc.)         │
│  ├── compliance/  (TfComplianceGuard)           │
│  └── ports/       (Traits — kontrak)            │
├─────────────────────────────────────────────────┤
│  crates/core/application/     ← Use Cases       │
│  └── services/ (SignalEngineService, Backtest)  │
├─────────────────────────────────────────────────┤
│  crates/adapters/             ← I/O Konkret     │
│  ├── storage-db/              (TimescaleDB)     │
│  ├── publisher-traderfamily/  (TF API)          │
│  ├── broker-connector/        (MT5/cTrader)     │
│  └── notifier-telegram/                         │
├─────────────────────────────────────────────────┤
│  crates/scrapers/             ← Data Sources    │
│  ├── dukascopy-rs/            (True Tick)       │
│  ├── forexfactory-rs/                           │
│  ├── myfxbook-rs/                               │
│  └── investing-rs/                              │
├─────────────────────────────────────────────────┤
│  apps/                        ← Binaries        │
│  ├── signal-daemon/           (Composition Root)│
│  └── cli/                                       │
└─────────────────────────────────────────────────┘
```

---

## 2. Interface-First / Trait-Driven Development

**Urutan wajib:**
1. Tulis `Port` (Trait) di `crates/core/domain/src/ports/`
2. Tulis mock implementasi untuk unit test (`InMemoryStorage`, dll.)
3. Tulis adapter konkret di `crates/adapters/`

```rust
// crates/core/domain/src/ports/signal_publisher.rs
#[async_trait]
pub trait SignalPublisherPort: Send + Sync {
    async fn publish(&self, signal: &Signal) -> Result<PublishReceipt, DomainError>;
    async fn cancel(&self, signal_id: SignalId) -> Result<(), DomainError>;
}
```

---

## 3. Composition over Inheritance

Rust tidak punya inheritance. Semua disusun via komposisi:

```rust
// BENAR: Komposisi
pub struct PolaNStrategy {
    pub swing_detector: SwingPointDetector,
    pub formation_engine: PolaNFormationEngine,
}

// SALAH: Tidak bisa, Rust tidak support class inheritance
// class PolaNStrategy extends BaseStrategy { ... }
```

---

## 4. Pure Dependency Injection — Composition Root

**DILARANG**: Global state / Singleton untuk klien jaringan atau database.

```rust
// DILARANG
static DB_CLIENT: Lazy<DbClient> = Lazy::new(|| DbClient::new());

// WAJIB: Constructor injection di Composition Root (apps/signal-daemon/src/main.rs)
let broker   = Arc::new(BrokerConnector::new(&config.broker).await?);
let publisher = Arc::new(TraderFamilyPublisher::new(&config.tf).await?);
let storage  = Arc::new(SqlxStorage::connect(&config.database).await?);

let engine = SignalEngineService::new(
    Arc::clone(&broker),
    vec![publisher as Arc<dyn SignalPublisherPort>],
    Arc::clone(&storage),
    strategies,
    risk_config,
);
```

---

## 5. Arsitektur Database: Unified Embedded WAL Storage

Sistem menggunakan SQLite berkemampuan tinggi dengan mode WAL (`Write-Ahead Logging`), synchronous normal, dan batch transaction atomic:

| Komponen | Implementasi | Fungsi & Karakteristik |
|:---|:---|:---|
| **Production Persistence Engine** | `SqliteStorage` (`storage-db/src/sqlite.rs`) | Mengimplementasikan `StoragePort`, `TickStoragePort`, dan `BrokerSpecPort`. Zero network latency, WAL mode, indexing B-Tree teroptimasi untuk query rentang waktu bar dan tick. |
| **Market Data Bridge** | `StorageMarketDataAdapter` (`storage-db/src/market_adapter.rs`) | Menyatukan `StoragePort` + `TickStoragePort` + `BrokerSpecPort` menjadi implementasi terpadu `MarketDataPort` untuk `BacktestService`, `Router`, dan API Handlers. |
| **Testing / Isolation Double** | `InMemoryStorage` (`storage-db/src/lib.rs`) | Penyimpanan in-memory hermetis berbasis `RwLock` untuk unit testing cepat dan terisolasi tanpa file disk. |
| **Ingestion Pipeline** | `MarketIngestionService` (`application/src/services/ingestion/`) | Idempotent seeding dari file mentah ke database SQLite dan live hot-ingestion via streaming socket. |

---

## 6. Arsitektur Frontend: Svelte 5 + Hexagonal

`ui/` mengadopsi pola yang sama:

- **Ports**: `ui/src/ports/` — interface TypeScript untuk setiap kapabilitas eksternal
- **Adapters**: Implementasi konkret (REST, WebSocket, LocalStorage)
- **Composition Root**: `ui/src/index.ts` — wiring semua dependensi
- **Domain**: `ui/src/domain/` — matematika trading murni, zero DOM/fetch dependency

**Stack**: Svelte 5 (Runes) + Vite + Tailwind CSS v4 + TradingView Lightweight Charts v5
