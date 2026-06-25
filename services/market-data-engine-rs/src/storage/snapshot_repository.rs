use sqlx::{Pool, Postgres};
use crate::events::DistributionSnapshot;

pub struct SnapshotRepository {
    pool: Pool<Postgres>,
}

impl SnapshotRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save_distribution_snapshot(&self, snapshot: &DistributionSnapshot) -> Result<(), sqlx::Error> {
        let json_data = serde_json::to_value(snapshot).map_err(|_| sqlx::Error::Protocol("JSON serialization error".into()))?;

        sqlx::query(
            r#"
            INSERT INTO distribution_snapshots (id, timestamp, payload)
            VALUES ($1, $2, $3)
            "#
        )
        .bind(&snapshot.id)
        .bind(snapshot.timestamp.timestamp_millis())
        .bind(&json_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
