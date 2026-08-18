-- ==============================================================================
-- 001_initial_schema.sql: TimescaleDB / PostgreSQL Forex Database Schema
-- ==============================================================================

-- 1. Tabel Candlestick (Time-Series Hypertable)
CREATE TABLE IF NOT EXISTS candles (
    symbol VARCHAR(10) NOT NULL,
    timeframe VARCHAR(10) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    open NUMERIC(12, 5) NOT NULL,
    high NUMERIC(12, 5) NOT NULL,
    low NUMERIC(12, 5) NOT NULL,
    close NUMERIC(12, 5) NOT NULL,
    volume NUMERIC(14, 2) NOT NULL,
    PRIMARY KEY (symbol, timeframe, timestamp)
);

-- Mengonversi tabel candles menjadi TimescaleDB Hypertable (jika ekstensi timescale tersedia)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        PERFORM create_hypertable('candles', 'timestamp', if_not_exists => TRUE);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_candles_lookup ON candles (symbol, timeframe, timestamp DESC);

-- 2. Tabel Sinyal Kuantitatif (Signals)
CREATE TABLE IF NOT EXISTS signals (
    id UUID PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    action VARCHAR(20) NOT NULL,
    timeframe VARCHAR(10) NOT NULL,
    entry_price NUMERIC(12, 5) NOT NULL,
    stop_loss NUMERIC(12, 5) NOT NULL,
    take_profit_1 NUMERIC(12, 5) NOT NULL,
    take_profit_2 NUMERIC(12, 5),
    take_profit_3 NUMERIC(12, 5),
    risk_reward_ratio NUMERIC(6, 2) NOT NULL,
    confidence_score REAL NOT NULL,
    strategy_name VARCHAR(100) NOT NULL,
    rationale TEXT NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'Active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_signals_status ON signals (status, created_at DESC);

-- 3. Tabel Order & Eksekusi Broker
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    action VARCHAR(20) NOT NULL,
    volume_lots NUMERIC(8, 2) NOT NULL,
    open_price NUMERIC(12, 5) NOT NULL,
    current_price NUMERIC(12, 5) NOT NULL,
    stop_loss NUMERIC(12, 5) NOT NULL,
    take_profit NUMERIC(12, 5) NOT NULL,
    open_time TIMESTAMPTZ NOT NULL,
    close_time TIMESTAMPTZ,
    realized_pnl NUMERIC(12, 2)
);

-- 4. Tabel Log Publikasi Channel (Trader Family / Telegram)
CREATE TABLE IF NOT EXISTS channel_publications (
    id SERIAL PRIMARY KEY,
    signal_id UUID NOT NULL REFERENCES signals(id) ON DELETE CASCADE,
    channel_platform VARCHAR(50) NOT NULL,
    channel_id VARCHAR(100) NOT NULL,
    external_post_id VARCHAR(100) NOT NULL,
    subscriber_count INT NOT NULL DEFAULT 0,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pub_signal ON channel_publications (signal_id);
