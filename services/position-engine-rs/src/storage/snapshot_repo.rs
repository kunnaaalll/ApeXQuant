use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub snapshot_id: i64,
    pub position_id: Uuid,
    pub snapshot_data: serde_json::Value,
    pub created_at: OffsetDateTime,
}

pub struct SnapshotRepository {
    pool: PgPool,
}

impl SnapshotRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_snapshot(
        &self,
        position_id: Uuid,
        data: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO position_snapshots (position_id, snapshot_data)
            VALUES ($1, $2)
            "#,
        )
        .bind(position_id)
        .bind(data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_latest_snapshot(
        &self,
        position_id: Uuid,
    ) -> Result<Option<SnapshotRecord>, sqlx::Error> {
        let row_opt = sqlx::query(
            r#"
            SELECT snapshot_id, position_id, snapshot_data, created_at
            FROM position_snapshots
            WHERE position_id = $1
            ORDER BY snapshot_id DESC
            LIMIT 1
            "#,
        )
        .bind(position_id)
        .fetch_optional(&self.pool)
        .await?;

        let record = row_opt.map(|row| SnapshotRecord {
            snapshot_id: row.get("snapshot_id"),
            position_id: row.get("position_id"),
            snapshot_data: row.get("snapshot_data"),
            created_at: row.get("created_at"),
        });

        Ok(record)
    }
}
