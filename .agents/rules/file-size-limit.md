---
description: >-
  Aturan batas maksimal panjang file — tidak boleh ada file melebihi 600 baris.
  File yang mendekati atau melebihi batas ini harus dipecah secara modular.
globs: "**/*.rs"
alwaysApply: true
---

# File Size Limit Rule — Maks 600 Baris

## Aturan

Tidak boleh ada **satu file** yang melebihi **600 baris kode** di workspace ini.

## Strategi Pemecahan

Jika file mendekati atau melebihi 600 baris:

1. **Domain Models**: Pisahkan ke sub-file per entitas (`entities/signal.rs`, `entities/order.rs`)
2. **Services**: Pisahkan logika besar ke modul helper (`services/backtest/fill_engine.rs`)
3. **Tests**: Pindahkan ke `tests.rs` atau direktori `tests/`
4. **Handlers**: Pisahkan per resource ke `handlers/signal.rs`, `handlers/account.rs`

## Contoh Struktur yang Baik

```
crates/core/domain/src/
├── entities/
│   ├── mod.rs          # re-export saja
│   ├── signal.rs       # < 200 baris
│   ├── order.rs        # < 200 baris
│   └── candle.rs       # < 150 baris
└── strategies/
    ├── mod.rs
    ├── pola_n/
    │   ├── mod.rs
    │   ├── swing_detector.rs
    │   └── formation_engine.rs
    └── ...
```
