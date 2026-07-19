use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// ─────────────────────────────────────────────────────────────────────────────
// Domain models persisted to PostgreSQL
// ─────────────────────────────────────────────────────────────────────────────

/// A single closed trade record as stored in the performance database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClosedTradeRecord {
    pub trade_id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub session: String,
    pub timeframe: String,
    pub pattern_id: String,
    pub direction: String, // "long" | "short"
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub sl_price: Decimal,
    pub tp_price: Decimal,
    pub rr: Decimal,
    pub r_outcome: Decimal, // P&L in R-multiples
    pub pnl_usd: Decimal,   // Absolute P&L in account currency
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub commission: Decimal,
    pub swap: Decimal,
    pub is_win: bool,
    pub entry_quality: Decimal,
    pub duration_seconds: i64,
    pub opened_at: chrono::DateTime<chrono::Utc>,
    pub closed_at: chrono::DateTime<chrono::Utc>,
}

/// Computed strategy performance metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StrategyMetricsSnapshot {
    pub snapshot_id: String,
    pub strategy_id: String,
    pub computed_at: chrono::DateTime<chrono::Utc>,
    pub trade_count: i64,
    pub win_count: i64,
    pub loss_count: i64,
    pub breakeven_count: i64,
    pub win_rate: Decimal,
    pub loss_rate: Decimal,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub net_profit: Decimal,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_rr: Decimal,
    pub max_drawdown: Decimal,
    pub average_drawdown: Decimal,
    pub recovery_factor: Decimal,
    pub ulcer_index: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub omega_ratio: Decimal,
    pub sqn: Decimal,
    pub max_consecutive_wins: i32,
    pub max_consecutive_losses: i32,
    pub health_score: i16,
    pub edge_score: Decimal,
    pub confidence: Decimal,
    pub stability: Decimal,
}

/// Degradation event logged when a strategy shows performance deterioration.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DegradationEvent {
    pub event_id: String,
    pub strategy_id: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub severity: Decimal,
    pub velocity: Decimal,
    pub edge_decay: Decimal,
    pub expectancy_decay: Decimal,
    pub performance_drift: Decimal,
    pub state: String,
    pub resolved: bool,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Overfitting detection result for a strategy configuration.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OverfitRecord {
    pub record_id: String,
    pub strategy_id: String,
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
    pub in_sample_trades: i32,
    pub out_of_sample_trades: i32,
    pub in_sample_expectancy: Decimal,
    pub out_of_sample_expectancy: Decimal,
    pub in_sample_pf: Decimal,
    pub out_of_sample_pf: Decimal,
    pub expectancy_ratio: Decimal,
    pub pf_ratio: Decimal,
    pub param_density: Decimal,
    pub confidence_penalty: Decimal,
    pub state: String,
    pub reasons: serde_json::Value,
}

/// A research task generated from weakness/opportunity analysis.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResearchJob {
    pub job_id: String,
    pub strategy_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub priority: i32,
    pub job_type: String, // "weakness", "opportunity", "overfitting", "degradation"
    pub dimension: String,
    pub label: String,
    pub description: String,
    pub status: String, // "pending", "in_progress", "completed", "cancelled"
    pub metadata: serde_json::Value,
}

// ─────────────────────────────────────────────────────────────────────────────
// DDL — run once at startup
// ─────────────────────────────────────────────────────────────────────────────

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS closed_trades (
    trade_id            TEXT PRIMARY KEY,
    strategy_id         TEXT NOT NULL,
    symbol              TEXT NOT NULL,
    session             TEXT NOT NULL DEFAULT '',
    timeframe           TEXT NOT NULL DEFAULT '',
    pattern_id          TEXT NOT NULL DEFAULT '',
    direction           TEXT NOT NULL DEFAULT 'long',
    entry_price         NUMERIC(20,8) NOT NULL DEFAULT 0,
    exit_price          NUMERIC(20,8) NOT NULL DEFAULT 0,
    sl_price            NUMERIC(20,8) NOT NULL DEFAULT 0,
    tp_price            NUMERIC(20,8) NOT NULL DEFAULT 0,
    rr                  NUMERIC(10,4) NOT NULL DEFAULT 0,
    r_outcome           NUMERIC(10,4) NOT NULL DEFAULT 0,
    pnl_usd             NUMERIC(20,8) NOT NULL DEFAULT 0,
    gross_profit        NUMERIC(20,8) NOT NULL DEFAULT 0,
    gross_loss          NUMERIC(20,8) NOT NULL DEFAULT 0,
    commission          NUMERIC(20,8) NOT NULL DEFAULT 0,
    swap                NUMERIC(20,8) NOT NULL DEFAULT 0,
    is_win              BOOLEAN NOT NULL DEFAULT FALSE,
    entry_quality       NUMERIC(5,4) NOT NULL DEFAULT 0,
    duration_seconds    BIGINT NOT NULL DEFAULT 0,
    opened_at           TIMESTAMPTZ NOT NULL,
    closed_at           TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_closed_trades_strategy ON closed_trades(strategy_id, closed_at DESC);
CREATE INDEX IF NOT EXISTS idx_closed_trades_symbol ON closed_trades(symbol, closed_at DESC);
CREATE INDEX IF NOT EXISTS idx_closed_trades_session ON closed_trades(session);

CREATE TABLE IF NOT EXISTS strategy_metrics_snapshots (
    snapshot_id             TEXT PRIMARY KEY,
    strategy_id             TEXT NOT NULL,
    computed_at             TIMESTAMPTZ NOT NULL,
    trade_count             BIGINT NOT NULL DEFAULT 0,
    win_count               BIGINT NOT NULL DEFAULT 0,
    loss_count              BIGINT NOT NULL DEFAULT 0,
    breakeven_count         BIGINT NOT NULL DEFAULT 0,
    win_rate                NUMERIC(8,6) NOT NULL DEFAULT 0,
    loss_rate               NUMERIC(8,6) NOT NULL DEFAULT 0,
    gross_profit            NUMERIC(20,8) NOT NULL DEFAULT 0,
    gross_loss              NUMERIC(20,8) NOT NULL DEFAULT 0,
    net_profit              NUMERIC(20,8) NOT NULL DEFAULT 0,
    profit_factor           NUMERIC(12,6) NOT NULL DEFAULT 0,
    expectancy              NUMERIC(12,6) NOT NULL DEFAULT 0,
    average_win             NUMERIC(20,8) NOT NULL DEFAULT 0,
    average_loss            NUMERIC(20,8) NOT NULL DEFAULT 0,
    largest_win             NUMERIC(20,8) NOT NULL DEFAULT 0,
    largest_loss            NUMERIC(20,8) NOT NULL DEFAULT 0,
    average_rr              NUMERIC(10,4) NOT NULL DEFAULT 0,
    max_drawdown            NUMERIC(8,6) NOT NULL DEFAULT 0,
    average_drawdown        NUMERIC(8,6) NOT NULL DEFAULT 0,
    recovery_factor         NUMERIC(12,6) NOT NULL DEFAULT 0,
    ulcer_index             NUMERIC(12,6) NOT NULL DEFAULT 0,
    sharpe_ratio            NUMERIC(12,6) NOT NULL DEFAULT 0,
    sortino_ratio           NUMERIC(12,6) NOT NULL DEFAULT 0,
    calmar_ratio            NUMERIC(12,6) NOT NULL DEFAULT 0,
    omega_ratio             NUMERIC(12,6) NOT NULL DEFAULT 0,
    sqn                     NUMERIC(12,6) NOT NULL DEFAULT 0,
    max_consecutive_wins    INTEGER NOT NULL DEFAULT 0,
    max_consecutive_losses  INTEGER NOT NULL DEFAULT 0,
    health_score            SMALLINT NOT NULL DEFAULT 0,
    edge_score              NUMERIC(8,6) NOT NULL DEFAULT 0,
    confidence              NUMERIC(8,6) NOT NULL DEFAULT 0,
    stability               NUMERIC(8,6) NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sms_strategy_time ON strategy_metrics_snapshots(strategy_id, computed_at DESC);

CREATE TABLE IF NOT EXISTS degradation_events (
    event_id            TEXT PRIMARY KEY,
    strategy_id         TEXT NOT NULL,
    detected_at         TIMESTAMPTZ NOT NULL,
    severity            NUMERIC(8,6) NOT NULL DEFAULT 0,
    velocity            NUMERIC(8,6) NOT NULL DEFAULT 0,
    edge_decay          NUMERIC(8,6) NOT NULL DEFAULT 0,
    expectancy_decay    NUMERIC(8,6) NOT NULL DEFAULT 0,
    performance_drift   NUMERIC(8,6) NOT NULL DEFAULT 0,
    state               TEXT NOT NULL,
    resolved            BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at         TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_degradation_strategy ON degradation_events(strategy_id, detected_at DESC);

CREATE TABLE IF NOT EXISTS overfit_records (
    record_id               TEXT PRIMARY KEY,
    strategy_id             TEXT NOT NULL,
    evaluated_at            TIMESTAMPTZ NOT NULL,
    in_sample_trades        INTEGER NOT NULL DEFAULT 0,
    out_of_sample_trades    INTEGER NOT NULL DEFAULT 0,
    in_sample_expectancy    NUMERIC(12,6) NOT NULL DEFAULT 0,
    out_of_sample_expectancy NUMERIC(12,6) NOT NULL DEFAULT 0,
    in_sample_pf            NUMERIC(12,6) NOT NULL DEFAULT 0,
    out_of_sample_pf        NUMERIC(12,6) NOT NULL DEFAULT 0,
    expectancy_ratio        NUMERIC(12,6) NOT NULL DEFAULT 0,
    pf_ratio                NUMERIC(12,6) NOT NULL DEFAULT 0,
    param_density           NUMERIC(12,6) NOT NULL DEFAULT 0,
    confidence_penalty      NUMERIC(8,6) NOT NULL DEFAULT 0,
    state                   TEXT NOT NULL,
    reasons                 JSONB NOT NULL DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_overfit_strategy ON overfit_records(strategy_id, evaluated_at DESC);

CREATE TABLE IF NOT EXISTS research_jobs (
    job_id          TEXT PRIMARY KEY,
    strategy_id     TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    priority        INTEGER NOT NULL DEFAULT 0,
    job_type        TEXT NOT NULL,
    dimension       TEXT NOT NULL,
    label           TEXT NOT NULL,
    description     TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    metadata        JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_research_strategy ON research_jobs(strategy_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_research_status ON research_jobs(status, priority DESC);
"#;

// ─────────────────────────────────────────────────────────────────────────────
// PerformanceRepository
// ─────────────────────────────────────────────────────────────────────────────

pub struct PerformanceRepository {
    pool: PgPool,
}

impl PerformanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run DDL migrations (idempotent — CREATE IF NOT EXISTS).
    pub async fn migrate(&self) -> Result<()> {
        for stmt in DDL.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt)
                .execute(&self.pool)
                .await
                .context("Failed to execute DDL migration")?;
        }
        Ok(())
    }

    /// Upsert a closed trade record. Idempotent on trade_id.
    pub async fn upsert_trade(&self, trade: &ClosedTradeRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO closed_trades (
                trade_id, strategy_id, symbol, session, timeframe, pattern_id,
                direction, entry_price, exit_price, sl_price, tp_price, rr,
                r_outcome, pnl_usd, gross_profit, gross_loss, commission, swap,
                is_win, entry_quality, duration_seconds, opened_at, closed_at
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,
                $13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23
            )
            ON CONFLICT (trade_id) DO UPDATE SET
                pnl_usd = EXCLUDED.pnl_usd,
                gross_profit = EXCLUDED.gross_profit,
                gross_loss = EXCLUDED.gross_loss,
                r_outcome = EXCLUDED.r_outcome,
                is_win = EXCLUDED.is_win
            "#,
        )
        .bind(&trade.trade_id)
        .bind(&trade.strategy_id)
        .bind(&trade.symbol)
        .bind(&trade.session)
        .bind(&trade.timeframe)
        .bind(&trade.pattern_id)
        .bind(&trade.direction)
        .bind(trade.entry_price)
        .bind(trade.exit_price)
        .bind(trade.sl_price)
        .bind(trade.tp_price)
        .bind(trade.rr)
        .bind(trade.r_outcome)
        .bind(trade.pnl_usd)
        .bind(trade.gross_profit)
        .bind(trade.gross_loss)
        .bind(trade.commission)
        .bind(trade.swap)
        .bind(trade.is_win)
        .bind(trade.entry_quality)
        .bind(trade.duration_seconds)
        .bind(trade.opened_at)
        .bind(trade.closed_at)
        .execute(&self.pool)
        .await
        .context("Failed to upsert closed trade")?;
        Ok(())
    }

    /// Fetch all closed trades for a strategy, ordered by close time ascending.
    pub async fn get_trades_for_strategy(
        &self,
        strategy_id: &str,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<ClosedTradeRecord>> {
        let from = from.unwrap_or(chrono::DateTime::UNIX_EPOCH);
        let to = to.unwrap_or_else(chrono::Utc::now);
        let rows = sqlx::query_as::<_, ClosedTradeRecord>(
            r#"
            SELECT * FROM closed_trades
            WHERE strategy_id = $1
              AND closed_at BETWEEN $2 AND $3
            ORDER BY closed_at ASC
            "#,
        )
        .bind(strategy_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch trades for strategy")?;
        Ok(rows)
    }

    /// Aggregate trade statistics grouped by a dimension (symbol, session, timeframe).
    pub async fn aggregate_by_dimension(
        &self,
        strategy_id: &str,
        dimension: &str,
    ) -> Result<Vec<DimensionAggregate>> {
        let col = match dimension {
            "symbol" => "symbol",
            "session" => "session",
            "timeframe" => "timeframe",
            "pattern" => "pattern_id",
            _ => "symbol",
        };

        let sql = format!(
            r#"
            SELECT
                {col} AS group_key,
                COUNT(*)::BIGINT AS trade_count,
                SUM(CASE WHEN is_win THEN 1 ELSE 0 END)::BIGINT AS wins,
                SUM(CASE WHEN NOT is_win THEN 1 ELSE 0 END)::BIGINT AS losses,
                SUM(pnl_usd) AS net_pnl,
                SUM(gross_profit) AS gross_profit,
                SUM(gross_loss) AS gross_loss,
                AVG(r_outcome) AS avg_r,
                MAX(r_outcome) AS max_win,
                MIN(r_outcome) AS max_loss
            FROM closed_trades
            WHERE strategy_id = $1
            GROUP BY {col}
            ORDER BY net_pnl DESC
            "#
        );

        let rows = sqlx::query_as::<_, DimensionAggregate>(&sql)
            .bind(strategy_id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to aggregate by dimension")?;
        Ok(rows)
    }

    /// Fetch trades for equity curve reconstruction, ordered by closed_at.
    pub async fn get_r_outcomes_for_strategy(
        &self,
        strategy_id: &str,
    ) -> Result<Vec<(chrono::DateTime<chrono::Utc>, Decimal, String)>> {
        let rows: Vec<(chrono::DateTime<chrono::Utc>, Decimal, String)> = sqlx::query_as(
            "SELECT closed_at, r_outcome, trade_id FROM closed_trades WHERE strategy_id = $1 ORDER BY closed_at ASC",
        )
        .bind(strategy_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch R-outcomes")?;
        Ok(rows)
    }

    /// Monthly aggregation for monthly returns report.
    pub async fn get_monthly_returns(
        &self,
        strategy_id: &str,
        year: i32,
    ) -> Result<Vec<MonthlyAgg>> {
        let rows = sqlx::query_as::<_, MonthlyAgg>(
            r#"
            SELECT
                TO_CHAR(closed_at, 'YYYY-MM') AS month,
                COUNT(*)::BIGINT AS trade_count,
                SUM(pnl_usd) AS net_pnl,
                SUM(CASE WHEN is_win THEN 1 ELSE 0 END)::BIGINT AS wins,
                SUM(CASE WHEN NOT is_win THEN 1 ELSE 0 END)::BIGINT AS losses
            FROM closed_trades
            WHERE strategy_id = $1
              AND EXTRACT(YEAR FROM closed_at) = $2
            GROUP BY TO_CHAR(closed_at, 'YYYY-MM')
            ORDER BY month ASC
            "#,
        )
        .bind(strategy_id)
        .bind(year as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch monthly returns")?;
        Ok(rows)
    }

    /// Check database health via a lightweight query.
    pub async fn health_check(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.pool).await.is_ok()
    }
}

/// Aggregated stats for a single dimension group.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DimensionAggregate {
    pub group_key: String,
    pub trade_count: i64,
    pub wins: i64,
    pub losses: i64,
    pub net_pnl: Decimal,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub avg_r: Decimal,
    pub max_win: Decimal,
    pub max_loss: Decimal,
}

/// Monthly aggregation row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MonthlyAgg {
    pub month: String,
    pub trade_count: i64,
    pub net_pnl: Decimal,
    pub wins: i64,
    pub losses: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// StatisticsRepository
// ─────────────────────────────────────────────────────────────────────────────

pub struct StatisticsRepository {
    pool: PgPool,
}

impl StatisticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert a full strategy metrics snapshot.
    pub async fn upsert_snapshot(&self, snap: &StrategyMetricsSnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO strategy_metrics_snapshots (
                snapshot_id, strategy_id, computed_at, trade_count,
                win_count, loss_count, breakeven_count, win_rate, loss_rate,
                gross_profit, gross_loss, net_profit, profit_factor, expectancy,
                average_win, average_loss, largest_win, largest_loss, average_rr,
                max_drawdown, average_drawdown, recovery_factor, ulcer_index,
                sharpe_ratio, sortino_ratio, calmar_ratio, omega_ratio, sqn,
                max_consecutive_wins, max_consecutive_losses,
                health_score, edge_score, confidence, stability
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,
                $15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,
                $29,$30,$31,$32,$33,$34
            )
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&snap.snapshot_id)
        .bind(&snap.strategy_id)
        .bind(snap.computed_at)
        .bind(snap.trade_count)
        .bind(snap.win_count)
        .bind(snap.loss_count)
        .bind(snap.breakeven_count)
        .bind(snap.win_rate)
        .bind(snap.loss_rate)
        .bind(snap.gross_profit)
        .bind(snap.gross_loss)
        .bind(snap.net_profit)
        .bind(snap.profit_factor)
        .bind(snap.expectancy)
        .bind(snap.average_win)
        .bind(snap.average_loss)
        .bind(snap.largest_win)
        .bind(snap.largest_loss)
        .bind(snap.average_rr)
        .bind(snap.max_drawdown)
        .bind(snap.average_drawdown)
        .bind(snap.recovery_factor)
        .bind(snap.ulcer_index)
        .bind(snap.sharpe_ratio)
        .bind(snap.sortino_ratio)
        .bind(snap.calmar_ratio)
        .bind(snap.omega_ratio)
        .bind(snap.sqn)
        .bind(snap.max_consecutive_wins)
        .bind(snap.max_consecutive_losses)
        .bind(snap.health_score)
        .bind(snap.edge_score)
        .bind(snap.confidence)
        .bind(snap.stability)
        .execute(&self.pool)
        .await
        .context("Failed to upsert statistics snapshot")?;
        Ok(())
    }

    /// Get the latest snapshot for a strategy.
    pub async fn get_latest_snapshot(
        &self,
        strategy_id: &str,
    ) -> Result<Option<StrategyMetricsSnapshot>> {
        let row = sqlx::query_as::<_, StrategyMetricsSnapshot>(
            r#"
            SELECT * FROM strategy_metrics_snapshots
            WHERE strategy_id = $1
            ORDER BY computed_at DESC
            LIMIT 1
            "#,
        )
        .bind(strategy_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch latest snapshot")?;
        Ok(row)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DegradationRepository
// ─────────────────────────────────────────────────────────────────────────────

pub struct DegradationRepository {
    pool: PgPool,
}

impl DegradationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_event(&self, ev: &DegradationEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO degradation_events (
                event_id, strategy_id, detected_at, severity, velocity,
                edge_decay, expectancy_decay, performance_drift, state, resolved
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(&ev.event_id)
        .bind(&ev.strategy_id)
        .bind(ev.detected_at)
        .bind(ev.severity)
        .bind(ev.velocity)
        .bind(ev.edge_decay)
        .bind(ev.expectancy_decay)
        .bind(ev.performance_drift)
        .bind(&ev.state)
        .bind(ev.resolved)
        .execute(&self.pool)
        .await
        .context("Failed to insert degradation event")?;
        Ok(())
    }

    pub async fn get_active_events(&self, strategy_id: &str) -> Result<Vec<DegradationEvent>> {
        let rows = sqlx::query_as::<_, DegradationEvent>(
            "SELECT * FROM degradation_events WHERE strategy_id = $1 AND resolved = FALSE ORDER BY detected_at DESC",
        )
        .bind(strategy_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch active degradation events")?;
        Ok(rows)
    }

    pub async fn resolve_events(&self, strategy_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE degradation_events SET resolved = TRUE, resolved_at = NOW() WHERE strategy_id = $1 AND resolved = FALSE",
        )
        .bind(strategy_id)
        .execute(&self.pool)
        .await
        .context("Failed to resolve degradation events")?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OverfitRepository
// ─────────────────────────────────────────────────────────────────────────────

pub struct OverfitRepository {
    pool: PgPool,
}

impl OverfitRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_record(&self, rec: &OverfitRecord) -> Result<()> {
        let reasons_json =
            serde_json::to_value(&rec.reasons).unwrap_or(serde_json::Value::Array(vec![]));
        sqlx::query(
            r#"
            INSERT INTO overfit_records (
                record_id, strategy_id, evaluated_at, in_sample_trades, out_of_sample_trades,
                in_sample_expectancy, out_of_sample_expectancy, in_sample_pf, out_of_sample_pf,
                expectancy_ratio, pf_ratio, param_density, confidence_penalty, state, reasons
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            ON CONFLICT (record_id) DO NOTHING
            "#,
        )
        .bind(&rec.record_id)
        .bind(&rec.strategy_id)
        .bind(rec.evaluated_at)
        .bind(rec.in_sample_trades)
        .bind(rec.out_of_sample_trades)
        .bind(rec.in_sample_expectancy)
        .bind(rec.out_of_sample_expectancy)
        .bind(rec.in_sample_pf)
        .bind(rec.out_of_sample_pf)
        .bind(rec.expectancy_ratio)
        .bind(rec.pf_ratio)
        .bind(rec.param_density)
        .bind(rec.confidence_penalty)
        .bind(&rec.state)
        .bind(reasons_json)
        .execute(&self.pool)
        .await
        .context("Failed to upsert overfit record")?;
        Ok(())
    }

    pub async fn get_latest(&self, strategy_id: &str) -> Result<Option<OverfitRecord>> {
        let row = sqlx::query_as::<_, OverfitRecord>(
            "SELECT * FROM overfit_records WHERE strategy_id = $1 ORDER BY evaluated_at DESC LIMIT 1",
        )
        .bind(strategy_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch overfit record")?;
        Ok(row)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResearchRepository
// ─────────────────────────────────────────────────────────────────────────────

pub struct ResearchRepository {
    pool: PgPool,
}

impl ResearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue_job(&self, job: &ResearchJob) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO research_jobs (
                job_id, strategy_id, created_at, priority, job_type,
                dimension, label, description, status, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (job_id) DO NOTHING
            "#,
        )
        .bind(&job.job_id)
        .bind(&job.strategy_id)
        .bind(job.created_at)
        .bind(job.priority)
        .bind(&job.job_type)
        .bind(&job.dimension)
        .bind(&job.label)
        .bind(&job.description)
        .bind(&job.status)
        .bind(&job.metadata)
        .execute(&self.pool)
        .await
        .context("Failed to enqueue research job")?;
        Ok(())
    }

    pub async fn get_pending_jobs(&self, limit: i64) -> Result<Vec<ResearchJob>> {
        let rows = sqlx::query_as::<_, ResearchJob>(
            "SELECT * FROM research_jobs WHERE status = 'pending' ORDER BY priority DESC, created_at ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending research jobs")?;
        Ok(rows)
    }

    pub async fn mark_completed(&self, job_id: &str) -> Result<()> {
        sqlx::query("UPDATE research_jobs SET status = 'completed' WHERE job_id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await
            .context("Failed to mark research job completed")?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: Create all repositories from a single pool
// ─────────────────────────────────────────────────────────────────────────────

pub struct Repositories {
    pub performance: PerformanceRepository,
    pub statistics: StatisticsRepository,
    pub degradation: DegradationRepository,
    pub overfit: OverfitRepository,
    pub research: ResearchRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            performance: PerformanceRepository::new(pool.clone()),
            statistics: StatisticsRepository::new(pool.clone()),
            degradation: DegradationRepository::new(pool.clone()),
            overfit: OverfitRepository::new(pool.clone()),
            research: ResearchRepository::new(pool),
        }
    }

    pub async fn migrate(&self) -> Result<()> {
        self.performance.migrate().await
    }
}
