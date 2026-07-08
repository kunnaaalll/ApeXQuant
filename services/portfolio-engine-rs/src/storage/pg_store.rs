use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction, Row};
use super::events::EventRecord;
use super::snapshots::{SnapshotFrequency, SnapshotRecord};
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// PostgresPortfolioStore provides asynchronous, non-blocking CRUD operations
/// for appending events and snapshots with strict transaction safety.
#[derive(Clone)]
pub struct PostgresPortfolioStore {
    pool: PgPool,
}

impl PostgresPortfolioStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize tables in PostgreSQL
    pub async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_events (
                id UUID PRIMARY KEY,
                aggregate_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                payload JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                metadata JSONB,
                UNIQUE (aggregate_id, version)
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_snapshots (
                id UUID PRIMARY KEY,
                aggregate_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL,
                snapshot_type VARCHAR(100) NOT NULL,
                frequency VARCHAR(50) NOT NULL,
                payload JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_allocations (
                id UUID PRIMARY KEY,
                portfolio_id VARCHAR(255) NOT NULL,
                allocations JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_optimizations (
                id UUID PRIMARY KEY,
                portfolio_id VARCHAR(255) NOT NULL,
                method VARCHAR(100) NOT NULL,
                weights JSONB NOT NULL,
                expected_return NUMERIC(20, 8) NOT NULL,
                estimated_volatility NUMERIC(20, 8) NOT NULL,
                sharpe_ratio NUMERIC(20, 8) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_exposures (
                id UUID PRIMARY KEY,
                portfolio_id VARCHAR(255) NOT NULL,
                gross_exposure NUMERIC(20, 8) NOT NULL,
                net_exposure NUMERIC(20, 8) NOT NULL,
                long_exposure NUMERIC(20, 8) NOT NULL,
                short_exposure NUMERIC(20, 8) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_health (
                id UUID PRIMARY KEY,
                portfolio_id VARCHAR(255) NOT NULL,
                health_score INT NOT NULL,
                status VARCHAR(50) NOT NULL,
                breakdown JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_quality (
                id UUID PRIMARY KEY,
                portfolio_id VARCHAR(255) NOT NULL,
                quality_score NUMERIC(20, 8) NOT NULL,
                breakdown JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_correlations (
                id UUID PRIMARY KEY,
                portfolio_id VARCHAR(255) NOT NULL,
                window VARCHAR(50) NOT NULL,
                matrix_type VARCHAR(50) NOT NULL,
                identifiers JSONB NOT NULL,
                data JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Begin a new transaction for atomicity
    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>> {
        let tx = self.pool.begin().await?;
        Ok(tx)
    }

    /// Appends an event to the persistent storage immutably.
    pub async fn append_event(&self, tx: &mut Transaction<'_, Postgres>, event: &EventRecord) -> Result<()> {
        let payload_json = serde_json::to_value(&event.payload)?;
        
        sqlx::query(
            r#"
            INSERT INTO portfolio_events (id, aggregate_id, version, event_type, payload, timestamp, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(event.id)
        .bind(&event.aggregate_id)
        .bind(event.version)
        .bind(&event.event_type)
        .bind(payload_json)
        .bind(event.timestamp)
        .bind(&event.metadata)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Appends a snapshot to the persistent storage immutably.
    pub async fn append_snapshot(&self, tx: &mut Transaction<'_, Postgres>, snapshot: &SnapshotRecord) -> Result<()> {
        let payload_json = serde_json::to_value(&snapshot.payload)?;
        let freq_str = snapshot.frequency.to_string();

        sqlx::query(
            r#"
            INSERT INTO portfolio_snapshots (id, aggregate_id, version, snapshot_type, frequency, payload, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(snapshot.id)
        .bind(&snapshot.aggregate_id)
        .bind(snapshot.version)
        .bind(&snapshot.snapshot_type)
        .bind(freq_str)
        .bind(payload_json)
        .bind(snapshot.timestamp)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Loads all events for a specific aggregate ID, ordered by version ascending.
    pub async fn load_events(&self, aggregate_id: &str) -> Result<Vec<EventRecord>> {
        let records = sqlx::query(
            r#"
            SELECT id, aggregate_id, version, event_type, payload, timestamp, metadata
            FROM portfolio_events
            WHERE aggregate_id = $1
            ORDER BY version ASC
            "#
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(records.len());
        for rec in records {
            events.push(EventRecord {
                id: rec.try_get("id")?,
                aggregate_id: rec.try_get("aggregate_id")?,
                version: rec.try_get("version")?,
                event_type: rec.try_get("event_type")?,
                payload: serde_json::from_value(rec.try_get("payload")?)?,
                timestamp: rec.try_get("timestamp")?,
                metadata: rec.try_get("metadata")?,
            });
        }

        Ok(events)
    }

    pub async fn load_events_since_time(&self, aggregate_id: &str, since_time: OffsetDateTime) -> Result<Vec<EventRecord>> {
        let records = sqlx::query(
            r#"
            SELECT id, aggregate_id, version, event_type, payload, timestamp, metadata
            FROM portfolio_events
            WHERE aggregate_id = $1 AND timestamp > $2
            ORDER BY version ASC
            "#
        )
        .bind(aggregate_id)
        .bind(since_time)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(records.len());
        for rec in records {
            events.push(EventRecord {
                id: rec.try_get("id")?,
                aggregate_id: rec.try_get("aggregate_id")?,
                version: rec.try_get("version")?,
                event_type: rec.try_get("event_type")?,
                payload: serde_json::from_value(rec.try_get("payload")?)?,
                timestamp: rec.try_get("timestamp")?,
                metadata: rec.try_get("metadata")?,
            });
        }

        Ok(events)
    }

    /// Loads the latest snapshot for a given aggregate ID and frequency.
    pub async fn load_latest_snapshot(&self, aggregate_id: &str, frequency: SnapshotFrequency) -> Result<Option<SnapshotRecord>> {
        let freq_str = frequency.to_string();

        let rec = sqlx::query(
            r#"
            SELECT id, aggregate_id, version, snapshot_type, frequency, payload, timestamp
            FROM portfolio_snapshots
            WHERE aggregate_id = $1 AND frequency = $2
            ORDER BY version DESC
            LIMIT 1
            "#
        )
        .bind(aggregate_id)
        .bind(freq_str)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = rec {
            let freq_val: String = r.try_get("frequency")?;
            let freq = serde_json::from_value(serde_json::Value::String(freq_val))?;
            Ok(Some(SnapshotRecord {
                id: r.try_get("id")?,
                aggregate_id: r.try_get("aggregate_id")?,
                version: r.try_get("version")?,
                snapshot_type: r.try_get("snapshot_type")?,
                frequency: freq,
                payload: serde_json::from_value(r.try_get("payload")?)?,
                timestamp: r.try_get("timestamp")?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Loads a snapshot by an exact version.
    pub async fn load_snapshot_by_version(&self, aggregate_id: &str, version: i64) -> Result<Option<SnapshotRecord>> {
        let rec = sqlx::query(
            r#"
            SELECT id, aggregate_id, version, snapshot_type, frequency, payload, timestamp
            FROM portfolio_snapshots
            WHERE aggregate_id = $1 AND version = $2
            LIMIT 1
            "#
        )
        .bind(aggregate_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = rec {
            let freq_val: String = r.try_get("frequency")?;
            let freq = serde_json::from_value(serde_json::Value::String(freq_val))?;
            Ok(Some(SnapshotRecord {
                id: r.try_get("id")?,
                aggregate_id: r.try_get("aggregate_id")?,
                version: r.try_get("version")?,
                snapshot_type: r.try_get("snapshot_type")?,
                frequency: freq,
                payload: serde_json::from_value(r.try_get("payload")?)?,
                timestamp: r.try_get("timestamp")?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_allocation(&self, portfolio_id: &str, allocations: &serde_json::Value) -> Result<()> {
        let id = uuid::Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            r#"
            INSERT INTO portfolio_allocations (id, portfolio_id, allocations, timestamp)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(id)
        .bind(portfolio_id)
        .bind(allocations)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_optimization(
        &self,
        portfolio_id: &str,
        method: &str,
        weights: &serde_json::Value,
        expected_return: Decimal,
        estimated_volatility: Decimal,
        sharpe_ratio: Decimal,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            r#"
            INSERT INTO portfolio_optimizations (id, portfolio_id, method, weights, expected_return, estimated_volatility, sharpe_ratio, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(id)
        .bind(portfolio_id)
        .bind(method)
        .bind(weights)
        .bind(expected_return)
        .bind(estimated_volatility)
        .bind(sharpe_ratio)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_exposure(
        &self,
        portfolio_id: &str,
        gross_exposure: Decimal,
        net_exposure: Decimal,
        long_exposure: Decimal,
        short_exposure: Decimal,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            r#"
            INSERT INTO portfolio_exposures (id, portfolio_id, gross_exposure, net_exposure, long_exposure, short_exposure, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(id)
        .bind(portfolio_id)
        .bind(gross_exposure)
        .bind(net_exposure)
        .bind(long_exposure)
        .bind(short_exposure)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_health(
        &self,
        portfolio_id: &str,
        health_score: i32,
        status: &str,
        breakdown: &serde_json::Value,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            r#"
            INSERT INTO portfolio_health (id, portfolio_id, health_score, status, breakdown, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(id)
        .bind(portfolio_id)
        .bind(health_score)
        .bind(status)
        .bind(breakdown)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_quality(
        &self,
        portfolio_id: &str,
        quality_score: Decimal,
        breakdown: &serde_json::Value,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            r#"
            INSERT INTO portfolio_quality (id, portfolio_id, quality_score, breakdown, timestamp)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(id)
        .bind(portfolio_id)
        .bind(quality_score)
        .bind(breakdown)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_correlation(
        &self,
        portfolio_id: &str,
        window: &str,
        matrix_type: &str,
        identifiers: &serde_json::Value,
        data: &serde_json::Value,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            r#"
            INSERT INTO portfolio_correlations (id, portfolio_id, window, matrix_type, identifiers, data, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(id)
        .bind(portfolio_id)
        .bind(window)
        .bind(matrix_type)
        .bind(identifiers)
        .bind(data)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
