//! Storage Module
//!
//! Persistence for simulation events, snapshots, reports, and parameter sets.
//! Provides a `PostgresStorageEngine` production implementation using `sqlx`.

use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub session_id: String,
    pub timestamp: OffsetDateTime,
    pub state_snapshot: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Abstract storage engine.
pub trait StorageEngine: Send + Sync {
    fn save_checkpoint(
        &self,
        checkpoint: Checkpoint,
    ) -> impl std::future::Future<Output = Result<(), StorageError>> + Send;
    fn load_checkpoint(
        &self,
        session_id: &str,
        timestamp: OffsetDateTime,
    ) -> impl std::future::Future<Output = Result<Option<Checkpoint>, StorageError>> + Send;
    fn save_offset(
        &self,
        session_id: &str,
        offset: i64,
    ) -> impl std::future::Future<Output = Result<(), StorageError>> + Send;
    fn get_offset(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<i64, StorageError>> + Send;
}

/// Production storage engine for backtester service backed by PostgreSQL.
pub struct PostgresStorageEngine {
    pool: PgPool,
}

impl PostgresStorageEngine {
    /// Connects to PostgreSQL and ensures tables are present.
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Initialize schema (if not using sqlx migrations externally)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backtest_checkpoints (
                session_id TEXT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                state_snapshot BYTEA NOT NULL,
                PRIMARY KEY (session_id, timestamp)
            );
            
            CREATE TABLE IF NOT EXISTS backtest_offsets (
                session_id TEXT PRIMARY KEY,
                offset_val BIGINT NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

impl StorageEngine for PostgresStorageEngine {
    async fn save_checkpoint(&self, checkpoint: Checkpoint) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO backtest_checkpoints (session_id, timestamp, state_snapshot)
            VALUES ($1, $2, $3)
            ON CONFLICT (session_id, timestamp) 
            DO UPDATE SET state_snapshot = EXCLUDED.state_snapshot
            "#,
        )
        .bind(&checkpoint.session_id)
        .bind(checkpoint.timestamp)
        .bind(&checkpoint.state_snapshot)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_checkpoint(
        &self,
        session_id: &str,
        timestamp: OffsetDateTime,
    ) -> Result<Option<Checkpoint>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT session_id, timestamp, state_snapshot 
            FROM backtest_checkpoints 
            WHERE session_id = $1 AND timestamp = $2
            "#,
        )
        .bind(session_id)
        .bind(timestamp)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            Ok(Some(Checkpoint {
                session_id: r.get("session_id"),
                timestamp: r.get("timestamp"),
                state_snapshot: r.get("state_snapshot"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn save_offset(&self, session_id: &str, offset: i64) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO backtest_offsets (session_id, offset_val)
            VALUES ($1, $2)
            ON CONFLICT (session_id) 
            DO UPDATE SET offset_val = EXCLUDED.offset_val
            "#,
        )
        .bind(session_id)
        .bind(offset)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_offset(&self, session_id: &str) -> Result<i64, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT offset_val FROM backtest_offsets WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            Ok(r.get("offset_val"))
        } else {
            Err(StorageError::NotFound(format!(
                "No offset found for session {}",
                session_id
            )))
        }
    }
}
