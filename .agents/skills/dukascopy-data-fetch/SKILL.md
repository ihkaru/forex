---
name: dukascopy-data-fetch
description: >-
  Use this skill when the user asks to download, fetch, decompress, or convert
  Dukascopy historical tick data (.bi5 files) to Parquet format for backtesting.
  Also use when setting up the dukascopy-rs scraper or troubleshooting data gaps.
---

# Dukascopy Data Fetch Skill

Sumber data historis resmi: **Dukascopy Bank SA** — True Tick sejak 2003, gratis.

## Format Data

- Input: `.bi5` (LZMA compressed binary tick data)
- Output: Apache Parquet (`data/historical/<PAIR>/<YEAR>/`)
- Schema: `timestamp_utc`, `bid: Decimal`, `ask: Decimal`, `bid_volume: Decimal`, `ask_volume: Decimal`

## Steps

### 1. Download Tick Data

```bash
# Download 1 bulan data untuk pair tertentu
cargo run -p dukascopy-rs -- fetch \
  --pair EURUSD \
  --from 2024-01-01 \
  --to 2024-01-31 \
  --output data/historical/

# Atau via CLI wrapper
./scripts/fetch_dukascopy.sh EURUSD 2024
```

### 2. Validasi Output Parquet

```bash
# Cek jumlah baris (harus > 1M baris per bulan untuk major pair)
duckdb -c "SELECT COUNT(*) FROM 'data/historical/EURUSD/2024/*.parquet'"

# Cek tidak ada gap > 5 menit di hari trading
duckdb -c "
SELECT timestamp_utc, 
       LAG(timestamp_utc) OVER (ORDER BY timestamp_utc) as prev_ts,
       timestamp_utc - LAG(timestamp_utc) OVER (ORDER BY timestamp_utc) as gap
FROM 'data/historical/EURUSD/2024/*.parquet'
HAVING gap > INTERVAL '5 minutes'
LIMIT 10
"
```

### 3. Cek Data Integrity

- Pastikan `Ask > Bid` di semua baris
- Spread maksimal 5.0 pips (ditolak `DataIntegrityValidator`)
- Timestamp sudah UTC (`DataIntegrityValidator` menolak selain UTC)

## Error Umum

| Error | Solusi |
|:---|:---|
| `lzma decompress failed` | File .bi5 korup — hapus dan download ulang |
| `0 rows` | Weekend / holiday — Dukascopy tidak menyediakan data di luar market hours |
| `gap detected > 5 min` | Normal di off-peak hours, cek apakah di sesi Asia |

## Validasi Akhir

```bash
cargo test -p dukascopy-rs -- --nocapture 2>&1 | tail -20
```
