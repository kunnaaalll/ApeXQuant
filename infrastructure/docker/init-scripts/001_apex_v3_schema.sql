-- ============================================================
-- APEX V3 — Phase 12: Complete PostgreSQL Schema
-- Run: psql -U apex -d apex_v3 -f 001_apex_v3_schema.sql
-- Idempotent: safe to run multiple times (IF NOT EXISTS throughout)
-- ============================================================

-- ============================================================
-- 1. EXTENSIONS
-- ============================================================
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- 2. EVENT BUS — Events store (core replay source)
-- ============================================================
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(255) NOT NULL,
    source VARCHAR(255) NOT NULL,
    topic VARCHAR(255) NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    payload BYTEA NOT NULL,
    payload_hash BYTEA NOT NULL,
    deduplication_key VARCHAR(255),
    causation_id VARCHAR(255),
    correlation_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_events_topic_time ON events (topic, occurred_at);
CREATE INDEX IF NOT EXISTS idx_events_source ON events (source);
CREATE UNIQUE INDEX IF NOT EXISTS idx_events_dedup ON events (deduplication_key) WHERE deduplication_key IS NOT NULL;

-- ============================================================
-- 3. EVENT BUS — Dead Letter Queue
-- ============================================================
CREATE TABLE IF NOT EXISTS dead_letter_queue (
    id UUID PRIMARY KEY,
    event_id UUID,
    consumer_group VARCHAR(255) NOT NULL,
    topic VARCHAR(255) NOT NULL,
    payload BYTEA NOT NULL,
    reason TEXT NOT NULL,
    error_details TEXT,
    failed_at TIMESTAMPTZ DEFAULT NOW(),
    retry_count INT DEFAULT 0
);

-- ============================================================
-- 4. MARKET DATA — Ticks
-- ============================================================
CREATE TABLE IF NOT EXISTS ticks (
    id              BIGSERIAL PRIMARY KEY,
    symbol          TEXT        NOT NULL,
    bid             NUMERIC(20, 8) NOT NULL,
    ask             NUMERIC(20, 8) NOT NULL,
    last            NUMERIC(20, 8),
    volume          NUMERIC(20, 8) DEFAULT 0,
    spread          NUMERIC(12, 8),
    timestamp_ms    BIGINT      NOT NULL,         -- unix millis, monotonic
    received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_ticks_symbol_ts ON ticks (symbol, timestamp_ms ASC);
CREATE INDEX IF NOT EXISTS idx_ticks_symbol       ON ticks (symbol, received_at DESC);

-- ============================================================
-- 5. MARKET DATA — Candles (OHLCV)
-- ============================================================
CREATE TABLE IF NOT EXISTS candles (
    id              BIGSERIAL PRIMARY KEY,
    symbol          TEXT        NOT NULL,
    timeframe       TEXT        NOT NULL,    -- e.g. "M1", "M5", "H1"
    open_price      NUMERIC(20, 8) NOT NULL,
    high_price      NUMERIC(20, 8) NOT NULL,
    low_price       NUMERIC(20, 8) NOT NULL,
    close_price     NUMERIC(20, 8) NOT NULL,
    volume          NUMERIC(20, 8) DEFAULT 0,
    tick_count      INT         NOT NULL DEFAULT 0,
    open_time       TIMESTAMPTZ NOT NULL,
    close_time      TIMESTAMPTZ NOT NULL,
    is_closed       BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_candles_symbol_tf_time ON candles (symbol, timeframe, open_time);
CREATE INDEX IF NOT EXISTS idx_candles_symbol      ON candles (symbol, close_time DESC);

-- ============================================================
-- 6. EXECUTION ENGINE — Order Lifecycle Events
-- ============================================================
CREATE TABLE IF NOT EXISTS execution_events (
    id              BIGSERIAL PRIMARY KEY,
    aggregate_id    UUID        NOT NULL,
    sequence_number INT         NOT NULL,
    event_type      TEXT        NOT NULL,    -- "OrderSubmitted","OrderFilled","OrderCancelled", etc.
    order_id        TEXT,
    position_id     TEXT,
    broker_ticket   TEXT,
    symbol          TEXT,
    side            TEXT,                    -- "Buy" | "Sell"
    order_type      TEXT,                    -- "Market" | "Limit" | "Stop"
    volume          NUMERIC(16, 4),
    requested_price NUMERIC(20, 8),
    fill_price      NUMERIC(20, 8),
    stop_loss       NUMERIC(20, 8),
    take_profit     NUMERIC(20, 8),
    slippage_points NUMERIC(12, 4),
    latency_ms      NUMERIC(12, 3),
    broker_retcode  INT,
    broker_comment  TEXT,
    pnl             NUMERIC(20, 8),
    swap            NUMERIC(20, 8),
    commission      NUMERIC(20, 8),
    version         INT         NOT NULL DEFAULT 1,
    payload         JSONB       NOT NULL DEFAULT '{}',
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_exec_events_agg_seq ON execution_events (aggregate_id, sequence_number);
CREATE INDEX IF NOT EXISTS idx_exec_events_order_id    ON execution_events (order_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_exec_events_position_id ON execution_events (position_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_exec_events_symbol      ON execution_events (symbol, occurred_at);
CREATE INDEX IF NOT EXISTS idx_exec_events_type        ON execution_events (event_type, occurred_at);

-- ============================================================
-- 7. PORTFOLIO — Position Snapshots
-- ============================================================
CREATE TABLE IF NOT EXISTS position_snapshots (
    id              BIGSERIAL PRIMARY KEY,
    snapshot_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source          TEXT        NOT NULL,    -- "broker" | "engine" | "replay"
    position_id     TEXT        NOT NULL,
    symbol          TEXT        NOT NULL,
    side            TEXT        NOT NULL,
    volume          NUMERIC(16, 4) NOT NULL,
    entry_price     NUMERIC(20, 8) NOT NULL,
    current_price   NUMERIC(20, 8),
    stop_loss       NUMERIC(20, 8),
    take_profit     NUMERIC(20, 8),
    floating_pnl    NUMERIC(20, 8),
    swap            NUMERIC(20, 8),
    commission      NUMERIC(20, 8)
);
CREATE INDEX IF NOT EXISTS idx_pos_snapshots_source ON position_snapshots (source, snapshot_at DESC);
CREATE INDEX IF NOT EXISTS idx_pos_snapshots_symbol ON position_snapshots (symbol, snapshot_at DESC);

-- ============================================================
-- 8. BROKER RECONCILIATION LOG
-- ============================================================
CREATE TABLE IF NOT EXISTS reconciliation_log (
    id              BIGSERIAL PRIMARY KEY,
    reconciled_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stage           TEXT        NOT NULL DEFAULT 'hourly',
    -- Broker state
    broker_balance          NUMERIC(20, 8),
    broker_equity           NUMERIC(20, 8),
    broker_margin           NUMERIC(20, 8),
    broker_free_margin      NUMERIC(20, 8),
    broker_position_count   INT,
    broker_order_count      INT,
    broker_trade_count      INT,
    -- Execution Engine state
    engine_position_count   INT,
    engine_order_count      INT,
    engine_trade_count      INT,
    -- Portfolio Engine state
    portfolio_position_count INT,
    -- PostgreSQL state
    pg_execution_event_count BIGINT,
    pg_position_count       INT,
    -- Drift indicators (TRUE = match, FALSE = drift detected)
    balance_match           BOOLEAN NOT NULL DEFAULT TRUE,
    equity_match            BOOLEAN NOT NULL DEFAULT TRUE,
    margin_match            BOOLEAN NOT NULL DEFAULT TRUE,
    position_count_match    BOOLEAN NOT NULL DEFAULT TRUE,
    order_count_match       BOOLEAN NOT NULL DEFAULT TRUE,
    trade_count_match       BOOLEAN NOT NULL DEFAULT TRUE,
    -- Overall
    all_match               BOOLEAN NOT NULL DEFAULT TRUE,
    drift_details           JSONB   NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_recon_log_at    ON reconciliation_log (reconciled_at DESC);
CREATE INDEX IF NOT EXISTS idx_recon_log_match ON reconciliation_log (all_match, reconciled_at DESC);

-- ============================================================
-- 9. REPLAY CERTIFICATION — State Hashes
-- ============================================================
CREATE TABLE IF NOT EXISTS replay_hashes (
    id              BIGSERIAL PRIMARY KEY,
    checkpoint_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    trade_count     INT         NOT NULL,   -- trade number at checkpoint (100, 200, ...)
    -- Original live state hashes (SHA-256 hex)
    portfolio_hash  TEXT        NOT NULL,
    positions_hash  TEXT        NOT NULL,
    risk_hash       TEXT        NOT NULL,
    events_hash     TEXT        NOT NULL,   -- hash of all event IDs in order up to this point
    -- Replay state hashes (populated after replay)
    replay_portfolio_hash   TEXT,
    replay_positions_hash   TEXT,
    replay_risk_hash        TEXT,
    replay_events_hash      TEXT,
    -- Verdict
    replay_completed        BOOLEAN NOT NULL DEFAULT FALSE,
    replay_completed_at     TIMESTAMPTZ,
    hashes_match            BOOLEAN,
    mismatch_detail         TEXT            -- NULL if match, detail of first mismatch otherwise
);
CREATE INDEX IF NOT EXISTS idx_replay_hashes_count  ON replay_hashes (trade_count);
CREATE INDEX IF NOT EXISTS idx_replay_hashes_match  ON replay_hashes (hashes_match, checkpoint_at);

-- ============================================================
-- 10. PHASE 12 — Trade Campaign Log
-- ============================================================
CREATE TABLE IF NOT EXISTS phase12_trades (
    id              BIGSERIAL PRIMARY KEY,
    trade_number    INT         NOT NULL,   -- sequential: 1 to 1000+
    stage           INT         NOT NULL,   -- 8, 9, or 10
    order_id        TEXT        NOT NULL,
    broker_ticket   TEXT,
    symbol          TEXT        NOT NULL,
    side            TEXT        NOT NULL,
    volume          NUMERIC(16, 4) NOT NULL,
    requested_price NUMERIC(20, 8),
    fill_price      NUMERIC(20, 8),
    close_price     NUMERIC(20, 8),
    slippage_points NUMERIC(12, 4),
    latency_ms      NUMERIC(12, 3),
    pnl             NUMERIC(20, 8),
    duration_secs   NUMERIC(12, 3),
    signal_strategy TEXT,
    signal_confidence INT,
    broker_retcode  INT,
    broker_reject   BOOLEAN     NOT NULL DEFAULT FALSE,
    recovery_event  BOOLEAN     NOT NULL DEFAULT FALSE,
    submitted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    filled_at       TIMESTAMPTZ,
    closed_at       TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'Submitted'  -- Submitted, Filled, Closed, Rejected, Error
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_p12_trades_num    ON phase12_trades (trade_number);
CREATE INDEX IF NOT EXISTS idx_p12_trades_symbol        ON phase12_trades (symbol, submitted_at);
CREATE INDEX IF NOT EXISTS idx_p12_trades_stage         ON phase12_trades (stage, status);
CREATE INDEX IF NOT EXISTS idx_p12_trades_order_id      ON phase12_trades (order_id);

-- ============================================================
-- 11. PHASE 12 — Recovery Events Log
-- ============================================================
CREATE TABLE IF NOT EXISTS phase12_recovery_events (
    id              BIGSERIAL PRIMARY KEY,
    stage           INT         NOT NULL,
    service_name    TEXT        NOT NULL,   -- "execution-engine", "event-bus", "postgres", etc.
    failure_type    TEXT        NOT NULL,   -- "restart", "network_partition", "redis_unavailable", etc.
    triggered_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    recovered_at    TIMESTAMPTZ,
    recovery_duration_ms BIGINT,
    positions_before INT,
    positions_after  INT,
    orphan_count     INT         NOT NULL DEFAULT 0,
    duplicate_count  INT         NOT NULL DEFAULT 0,
    recovered        BOOLEAN     NOT NULL DEFAULT FALSE,
    notes            TEXT
);
CREATE INDEX IF NOT EXISTS idx_p12_recovery_stage ON phase12_recovery_events (stage, triggered_at);

-- ============================================================
-- 12. PHASE 12 — Infrastructure Metrics Snapshots
-- ============================================================
CREATE TABLE IF NOT EXISTS phase12_infra_metrics (
    id              BIGSERIAL PRIMARY KEY,
    captured_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    service_name    TEXT        NOT NULL,
    cpu_percent     NUMERIC(8, 3),
    memory_mb       NUMERIC(12, 3),
    event_throughput_per_sec NUMERIC(12, 3),
    queue_depth     INT,
    consumer_lag    INT,
    dlq_size        INT,
    retry_count     INT,
    db_connections  INT,
    redis_connected BOOLEAN
);
CREATE INDEX IF NOT EXISTS idx_p12_infra_captured ON phase12_infra_metrics (captured_at DESC, service_name);

-- ============================================================
-- 13. SCHEMA VERSION TRACKING
-- ============================================================
CREATE TABLE IF NOT EXISTS schema_migrations (
    version     TEXT        PRIMARY KEY,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT
);

INSERT INTO schema_migrations (version, description)
VALUES ('001', 'APEX V3 Phase 12 — Complete schema: events, ticks, candles, execution, portfolio, reconciliation, replay, campaign')
ON CONFLICT (version) DO NOTHING;

-- ============================================================
-- GRANTS (ensure apex user has full access)
-- ============================================================
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO apex;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO apex;
