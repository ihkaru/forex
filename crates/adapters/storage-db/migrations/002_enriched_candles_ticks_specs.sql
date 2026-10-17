-- ==============================================================================
-- 002_enriched_candles_ticks_specs.sql: Enriched Candles, Ticks, and Broker Specs
-- ==============================================================================

-- 1. Tambah Kolom Spread & Real Volume pada Tabel Candlestick
ALTER TABLE candles ADD COLUMN IF NOT EXISTS spread NUMERIC(10, 5);
ALTER TABLE candles ADD COLUMN IF NOT EXISTS real_volume NUMERIC(14, 2);

-- 2. Tabel Tick Kompresi Tinggi (High-Density Time-Series Hypertable)
CREATE TABLE IF NOT EXISTS ticks (
    symbol VARCHAR(10) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    source VARCHAR(30) NOT NULL,
    bid NUMERIC(12, 5) NOT NULL,
    ask NUMERIC(12, 5) NOT NULL,
    spread_pts INT,
    volume NUMERIC(14, 2),
    PRIMARY KEY (symbol, source, timestamp)
);

-- Konversi ke Hypertable TimescaleDB jika ekstensi tersedia
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        PERFORM create_hypertable('ticks', 'timestamp', if_not_exists => TRUE);
        
        -- Aktifkan kompresi Gorilla / Delta-of-Delta sekelas Parquet / Vortex
        ALTER TABLE ticks SET (
            timescaledb.compress,
            timescaledb.compress_segmentby = 'symbol, source',
            timescaledb.compress_orderby = 'timestamp DESC'
        );
        PERFORM add_compression_policy('ticks', INTERVAL '1 day', if_not_exists => TRUE);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_ticks_lookup ON ticks (symbol, source, timestamp DESC);

-- 3. Tabel Spesifikasi Simbol Broker (Broker Contract Specifications)
CREATE TABLE IF NOT EXISTS broker_symbol_specs (
    symbol VARCHAR(10) PRIMARY KEY,
    raw_ticker VARCHAR(30) NOT NULL,
    digits INT NOT NULL,
    point NUMERIC(10, 6) NOT NULL,
    tick_value NUMERIC(10, 4) NOT NULL,
    tick_size NUMERIC(10, 6) NOT NULL,
    contract_size NUMERIC(14, 2) NOT NULL,
    swap_long NUMERIC(10, 4) NOT NULL,
    swap_short NUMERIC(10, 4) NOT NULL,
    swap_rollover_3days INT NOT NULL DEFAULT 3,
    stop_level_pts INT NOT NULL DEFAULT 0,
    freeze_level_pts INT NOT NULL DEFAULT 0,
    min_lot NUMERIC(8, 2) NOT NULL DEFAULT 0.01,
    lot_step NUMERIC(8, 2) NOT NULL DEFAULT 0.01,
    max_lot NUMERIC(8, 2) NOT NULL DEFAULT 100.0,
    margin_required NUMERIC(12, 2) NOT NULL DEFAULT 1000.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
