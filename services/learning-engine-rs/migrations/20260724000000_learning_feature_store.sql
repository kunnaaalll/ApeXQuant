-- Phase 1 supplement: training records and feature store persistence
-- These tables capture all consumed events for the feature store and training pipeline.

CREATE TABLE IF NOT EXISTS learning_event_records (
    record_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type       VARCHAR(100) NOT NULL,
    topic            VARCHAR(200) NOT NULL,
    strategy_id      VARCHAR(255) NOT NULL DEFAULT 'unknown',
    symbol           VARCHAR(50)  NOT NULL DEFAULT '',
    net_pnl          DECIMAL      NOT NULL DEFAULT 0,
    gross_pnl        DECIMAL      NOT NULL DEFAULT 0,
    label_is_winner  BOOLEAN,
    -- Feature payload as JSONB for flexibility
    features         JSONB        NOT NULL DEFAULT '{}',
    raw_payload      JSONB        NOT NULL DEFAULT '{}',
    recorded_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_learning_event_records_strategy
    ON learning_event_records (strategy_id, recorded_at DESC);

CREATE INDEX IF NOT EXISTS idx_learning_event_records_type
    ON learning_event_records (event_type, recorded_at DESC);

-- Instrument-level statistics
CREATE TABLE IF NOT EXISTS learning_instrument_stats (
    symbol           VARCHAR(50)  NOT NULL,
    timeframe        VARCHAR(20)  NOT NULL DEFAULT 'ANY',
    total_events     BIGINT       NOT NULL DEFAULT 0,
    total_trades     BIGINT       NOT NULL DEFAULT 0,
    winning_trades   BIGINT       NOT NULL DEFAULT 0,
    avg_pnl          DECIMAL      NOT NULL DEFAULT 0,
    ema_pnl          DECIMAL      NOT NULL DEFAULT 0,
    last_updated     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (symbol, timeframe)
);

-- Correlation data between strategies and instruments
CREATE TABLE IF NOT EXISTS learning_correlation_data (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_a       VARCHAR(255) NOT NULL,
    strategy_b       VARCHAR(255) NOT NULL,
    correlation      DECIMAL      NOT NULL,
    sample_count     BIGINT       NOT NULL DEFAULT 0,
    computed_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_learning_correlation_unique
    ON learning_correlation_data (strategy_a, strategy_b);

-- Model metadata and performance history
CREATE TABLE IF NOT EXISTS learning_model_metadata (
    model_id         VARCHAR(255) PRIMARY KEY,
    version          VARCHAR(50)  NOT NULL DEFAULT '0.0.1',
    state            VARCHAR(50)  NOT NULL DEFAULT 'pending',  -- pending, training, ready, retired
    training_samples BIGINT       NOT NULL DEFAULT 0,
    last_trained_at  TIMESTAMPTZ,
    metrics          JSONB        NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Performance history per model
CREATE TABLE IF NOT EXISTS learning_performance_history (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id         VARCHAR(255) NOT NULL,
    sharpe_ratio     DECIMAL,
    win_rate         DECIMAL,
    profit_factor    DECIMAL,
    max_drawdown     DECIMAL,
    total_trades     BIGINT       NOT NULL DEFAULT 0,
    period_start     TIMESTAMPTZ  NOT NULL,
    period_end       TIMESTAMPTZ  NOT NULL,
    recorded_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Strategy performance snapshots (hourly rollup)
CREATE TABLE IF NOT EXISTS learning_strategy_history (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id      VARCHAR(255) NOT NULL,
    total_trades     BIGINT       NOT NULL DEFAULT 0,
    win_rate         DECIMAL      NOT NULL DEFAULT 0,
    avg_pnl          DECIMAL      NOT NULL DEFAULT 0,
    max_drawdown     DECIMAL      NOT NULL DEFAULT 0,
    confidence_score DECIMAL      NOT NULL DEFAULT 0,
    decay_score      DECIMAL      NOT NULL DEFAULT 0,
    snapshot_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_learning_strategy_history_strategy
    ON learning_strategy_history (strategy_id, snapshot_at DESC);
