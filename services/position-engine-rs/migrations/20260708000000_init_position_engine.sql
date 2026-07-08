-- Table: positions
CREATE TABLE IF NOT EXISTS positions (
    position_id UUID PRIMARY KEY,
    symbol VARCHAR(50) NOT NULL,
    side VARCHAR(10) NOT NULL,
    state VARCHAR(20) NOT NULL,
    initial_volume NUMERIC(20, 8) NOT NULL,
    current_volume NUMERIC(20, 8) NOT NULL,
    entry_price NUMERIC(20, 8) NOT NULL,
    current_price NUMERIC(20, 8) NOT NULL,
    stop_loss NUMERIC(20, 8),
    take_profit NUMERIC(20, 8),
    unrealized_pnl NUMERIC(20, 8) NOT NULL DEFAULT 0,
    realized_pnl NUMERIC(20, 8) NOT NULL DEFAULT 0,
    margin_used NUMERIC(20, 8) NOT NULL DEFAULT 0,
    commission NUMERIC(20, 8) NOT NULL DEFAULT 0,
    swap NUMERIC(20, 8) NOT NULL DEFAULT 0,
    leverage NUMERIC(10, 2) NOT NULL DEFAULT 1,
    opened_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_positions_state ON positions (state);
CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions (symbol);

-- Table: position_events
CREATE TABLE IF NOT EXISTS position_events (
    sequence_id BIGSERIAL PRIMARY KEY,
    position_id UUID NOT NULL REFERENCES positions(position_id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_position_events_position_id ON position_events(position_id);

-- Table: position_snapshots
CREATE TABLE IF NOT EXISTS position_snapshots (
    snapshot_id BIGSERIAL PRIMARY KEY,
    position_id UUID NOT NULL REFERENCES positions(position_id) ON DELETE CASCADE,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_position_snapshots_position_id ON position_snapshots(position_id);

-- Table: position_analytics
CREATE TABLE IF NOT EXISTS position_analytics (
    position_id UUID PRIMARY KEY REFERENCES positions(position_id) ON DELETE CASCADE,
    holding_efficiency NUMERIC(20, 8) NOT NULL DEFAULT 0,
    time_efficiency NUMERIC(20, 8) NOT NULL DEFAULT 0,
    profit_velocity NUMERIC(20, 8) NOT NULL DEFAULT 0,
    drawdown_duration BIGINT NOT NULL DEFAULT 0,
    recovery_time BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table: position_health
CREATE TABLE IF NOT EXISTS position_health (
    position_id UUID PRIMARY KEY REFERENCES positions(position_id) ON DELETE CASCADE,
    health_score NUMERIC(10, 2) NOT NULL DEFAULT 0,
    margin_utilization NUMERIC(10, 2) NOT NULL DEFAULT 0,
    stop_distance NUMERIC(10, 2) NOT NULL DEFAULT 0,
    liquidation_distance NUMERIC(10, 2) NOT NULL DEFAULT 0,
    drawdown NUMERIC(10, 2) NOT NULL DEFAULT 0,
    age_score NUMERIC(10, 2) NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
