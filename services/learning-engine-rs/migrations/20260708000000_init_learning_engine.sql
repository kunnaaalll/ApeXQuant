CREATE TABLE IF NOT EXISTS learning_lessons (
    lesson_id UUID PRIMARY KEY,
    position_id VARCHAR(255) NOT NULL,
    signal_id VARCHAR(255) NOT NULL,
    strategy_id VARCHAR(255) NOT NULL,
    lesson_type VARCHAR(50) NOT NULL,
    category VARCHAR(50) NOT NULL,
    severity DOUBLE PRECISION NOT NULL,
    symbol VARCHAR(50) NOT NULL,
    market_regime VARCHAR(100) NOT NULL,
    gross_pnl DECIMAL NOT NULL,
    net_pnl DECIMAL NOT NULL,
    entry_efficiency DECIMAL NOT NULL,
    exit_efficiency DECIMAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learning_recommendations (
    id UUID PRIMARY KEY,
    strategy_id VARCHAR(255) NOT NULL,
    recommendation_type VARCHAR(50) NOT NULL,
    confidence DECIMAL NOT NULL,
    reasoning TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    applied BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS learning_memory (
    strategy_id VARCHAR(255) PRIMARY KEY,
    total_trades BIGINT NOT NULL DEFAULT 0,
    winning_trades BIGINT NOT NULL DEFAULT 0,
    regime_quality DECIMAL NOT NULL DEFAULT 1.0,
    execution_quality DECIMAL NOT NULL DEFAULT 1.0,
    ema_return DECIMAL NOT NULL DEFAULT 0.0,
    historical_sum_return DECIMAL NOT NULL DEFAULT 0.0,
    regime_transitions INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learning_features (
    feature_id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    importance DECIMAL NOT NULL,
    stability DECIMAL NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learning_training_runs (
    run_id UUID PRIMARY KEY,
    model_id VARCHAR(255) NOT NULL,
    state VARCHAR(50) NOT NULL,
    loss DOUBLE PRECISION,
    accuracy DOUBLE PRECISION,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
