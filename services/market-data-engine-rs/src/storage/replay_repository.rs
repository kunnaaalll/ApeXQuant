use sqlx::{Pool, Postgres};
use crate::replay::ReplaySnapshot;

pub struct ReplayRepository {
    pool: Pool<Postgres>,
}

impl ReplayRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save_replay_snapshot(&self, snapshot: &ReplaySnapshot) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO replay_snapshots (sequence_number, timestamp, state_hash, data_blob)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(snapshot.checkpoint.sequence_number as i64)
        .bind(snapshot.checkpoint.timestamp.timestamp_millis())
        .bind(&snapshot.checkpoint.state_hash)
        .bind(&snapshot.data_blob)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
