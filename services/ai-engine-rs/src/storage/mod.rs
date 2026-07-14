use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::PgPool;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub state_data: Vec<u8>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSourcingRecord {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub trail_id: Uuid,
    pub records: Vec<EventSourcingRecord>,
}

pub struct StorageEngine {
    pool: PgPool,
}

impl StorageEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_event(&self, record: &EventSourcingRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO ai_events (event_id, aggregate_id, event_type, payload, recorded_at)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(record.event_id)
        .bind(record.aggregate_id)
        .bind(&record.event_type)
        .bind(&record.payload)
        .bind(record.recorded_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
