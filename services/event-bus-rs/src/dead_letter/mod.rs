use anyhow::{Context, Result};
use apex_protos::events::Event;
use chrono::Utc;
use sqlx::PgPool;

#[derive(Clone)]
pub struct DlqManager {
    pool: PgPool,
}

impl DlqManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_dead_letter(&self, event: &Event, error_msg: &str) -> Result<()> {
        let payload = prost::Message::encode_to_vec(event);
        let event_id = uuid::Uuid::new_v4(); // Or extract from event if valid

        sqlx::query(
            r#"
            INSERT INTO dead_letters (id, topic, payload, error, occurred_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(event_id)
        .bind(&event.topic)
        .bind(&payload)
        .bind(error_msg)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .context("Failed to insert into dead letters table")?;

        Ok(())
    }
}
