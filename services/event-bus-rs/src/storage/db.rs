use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use apex_protos::events::Event;
use anyhow::{Result, Context};
use std::str::FromStr;

#[derive(Clone)]
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY,
                event_type VARCHAR(255) NOT NULL,
                source VARCHAR(255) NOT NULL,
                topic VARCHAR(255) NOT NULL,
                occurred_at TIMESTAMPTZ NOT NULL,
                published_at TIMESTAMPTZ NOT NULL,
                payload BYTEA NOT NULL,
                payload_hash BYTEA NOT NULL,
                deduplication_key VARCHAR(255),
                causation_id VARCHAR(255),
                correlation_id VARCHAR(255),
                created_at TIMESTAMPTZ DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_events_topic_time ON events (topic, occurred_at);
            CREATE INDEX IF NOT EXISTS idx_events_source ON events (source);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_events_dedup ON events (deduplication_key) WHERE deduplication_key IS NOT NULL;

            CREATE TABLE IF NOT EXISTS subscriber_offsets (
                consumer_group VARCHAR(255) NOT NULL,
                topic VARCHAR(255) NOT NULL,
                last_event_id UUID NOT NULL,
                last_occurred_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                PRIMARY KEY (consumer_group, topic)
            );

            CREATE TABLE IF NOT EXISTS dead_letter_queue (
                id UUID PRIMARY KEY,
                event_id UUID,
                consumer_group VARCHAR(255) NOT NULL,
                topic VARCHAR(255) NOT NULL,
                payload BYTEA NOT NULL,
                reason TEXT NOT NULL,
                error_details TEXT,
                failed_at TIMESTAMPTZ DEFAULT NOW(),
                retry_count INT DEFAULT 0
            );
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to initialize event store schema")?;

        Ok(())
    }

    pub async fn store_event(&self, event: &Event) -> Result<()> {
        let mut encoded = vec![];
        prost::Message::encode(event, &mut encoded)?;

        let event_id_str = event.event_id.as_ref().map(|id| String::from_utf8_lossy(&id.value).into_owned()).unwrap_or_default();
        let event_id = uuid::Uuid::from_str(&event_id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
        
        let occurred_at: DateTime<Utc> = event.occurred_at.as_ref()
            .map(|t| DateTime::<Utc>::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default())
            .unwrap_or_else(|| Utc::now());

        let published_at: DateTime<Utc> = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO events (
                id, event_type, source, topic, occurred_at, published_at, 
                payload, payload_hash, deduplication_key, causation_id, correlation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(event_id)
        .bind(&event.event_type)
        .bind(&event.source_service)
        .bind(&event.topic)
        .bind(occurred_at)
        .bind(published_at)
        .bind(&encoded)
        .bind(&event.payload_hash)
        .bind(if event.deduplication_key.is_empty() { None::<String> } else { Some(event.deduplication_key.clone()) })
        .bind(if event.causation_id.is_empty() { None::<String> } else { Some(event.causation_id.clone()) })
        .bind(event.correlation.as_ref().and_then(|c| c.trace_id.as_ref().map(|id| String::from_utf8_lossy(&id.value).into_owned())))
        .execute(&self.pool)
        .await
        .context("Failed to store event")?;

        Ok(())
    }
    pub async fn get_stream_stats(&self, topic: &str) -> Result<(i64, i64, Option<DateTime<Utc>>, Option<DateTime<Utc>>)> {
        let row: (Option<i64>, Option<i64>, Option<DateTime<Utc>>, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*)::bigint as total_events,
                SUM(OCTET_LENGTH(payload))::bigint as byte_size,
                MIN(occurred_at) as oldest_event,
                MAX(occurred_at) as newest_event
            FROM events
            WHERE topic = $1
            "#
        )
        .bind(topic)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get stream stats")?;

        Ok((row.0.unwrap_or(0), row.1.unwrap_or(0), row.2, row.3))
    }

    pub async fn update_subscriber_offset(&self, consumer_group: &str, topic: &str, last_event_id: uuid::Uuid, last_occurred_at: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO subscriber_offsets (consumer_group, topic, last_event_id, last_occurred_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (consumer_group, topic) 
            DO UPDATE SET 
                last_event_id = EXCLUDED.last_event_id,
                last_occurred_at = EXCLUDED.last_occurred_at,
                updated_at = NOW()
            "#
        )
        .bind(consumer_group)
        .bind(topic)
        .bind(last_event_id)
        .bind(last_occurred_at)
        .execute(&self.pool)
        .await
        .context("Failed to update subscriber offset")?;
        Ok(())
    }

    pub async fn get_subscriber_offset(&self, consumer_group: &str, topic: &str) -> Result<Option<(uuid::Uuid, DateTime<Utc>)>> {
        let row: Option<(uuid::Uuid, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT last_event_id, last_occurred_at
            FROM subscriber_offsets
            WHERE consumer_group = $1 AND topic = $2
            "#
        )
        .bind(consumer_group)
        .bind(topic)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get subscriber offset")?;
        
        Ok(row)
    }
}
