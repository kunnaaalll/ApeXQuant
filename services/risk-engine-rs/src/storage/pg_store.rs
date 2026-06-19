use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::storage::events::EventRecord;
use crate::storage::snapshots::SnapshotRecord;

pub struct PostgresRiskStore {
    pool: PgPool,
}

impl PostgresRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append_event(&self, event: &EventRecord) -> Result<(), sqlx::Error> {
        let payload_json = serde_json::to_value(&event.payload)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO risk_events (event_id, aggregate_id, sequence, timestamp, payload, version)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(event.event_id)
        .bind(event.aggregate_id)
        .bind(event.sequence)
        .bind(event.timestamp)
        .bind(payload_json)
        .bind(event.version)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn append_snapshot(&self, snapshot: &SnapshotRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO risk_snapshots (aggregate_id, version, timestamp, snapshot)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(snapshot.aggregate_id)
        .bind(snapshot.version)
        .bind(snapshot.timestamp)
        .bind(&snapshot.snapshot)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_events(&self, aggregate_id: Uuid, since_sequence: i64) -> Result<Vec<EventRecord>, sqlx::Error> {
        let records = sqlx::query(
            r#"
            SELECT event_id, aggregate_id, sequence, timestamp, payload, version
            FROM risk_events
            WHERE aggregate_id = $1 AND sequence > $2
            ORDER BY sequence ASC
            "#
        )
        .bind(aggregate_id)
        .bind(since_sequence)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for record in records {
            let payload_value: serde_json::Value = record.try_get("payload")?;
            let payload = serde_json::from_value(payload_value)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            
            events.push(EventRecord {
                event_id: record.try_get("event_id")?,
                aggregate_id: record.try_get("aggregate_id")?,
                sequence: record.try_get("sequence")?,
                timestamp: record.try_get("timestamp")?,
                payload,
                version: record.try_get("version")?,
            });
        }

        Ok(events)
    }

    pub async fn load_snapshot(&self, aggregate_id: Uuid) -> Result<Option<SnapshotRecord>, sqlx::Error> {
        let record_opt = sqlx::query(
            r#"
            SELECT aggregate_id, version, timestamp, snapshot
            FROM risk_snapshots
            WHERE aggregate_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = record_opt {
            Ok(Some(SnapshotRecord {
                aggregate_id: record.try_get("aggregate_id")?,
                version: record.try_get("version")?,
                timestamp: record.try_get("timestamp")?,
                snapshot: record.try_get("snapshot")?,
            }))
        } else {
            Ok(None)
        }
    }
}
