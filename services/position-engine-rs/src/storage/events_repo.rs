use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub sequence_id: i64,
    pub position_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: OffsetDateTime,
}

pub struct EventsRepository {
    pool: PgPool,
}

impl EventsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append_event(
        &self,
        position_id: Uuid,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO position_events (position_id, event_type, payload)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(position_id)
        .bind(event_type)
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_events(&self, position_id: Uuid) -> Result<Vec<EventRecord>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT sequence_id, position_id, event_type, payload, created_at
            FROM position_events
            WHERE position_id = $1
            ORDER BY sequence_id ASC
            "#,
        )
        .bind(position_id)
        .fetch_all(&self.pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|row| EventRecord {
                sequence_id: row.get("sequence_id"),
                position_id: row.get("position_id"),
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(events)
    }
}
