use sqlx::{Pool, Postgres, Transaction};
use crate::storage::StorageError;
use crate::storage::events::EventRecord;
use crate::storage::snapshots::SnapshotRecord;

pub struct PgStore {
    pool: Pool<Postgres>,
}

impl PgStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, StorageError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn append_event(
        tx: &mut Transaction<'_, Postgres>,
        event: &EventRecord,
    ) -> Result<(), sqlx::Error> {
        let payload_json = serde_json::to_value(&event.payload).unwrap();

        sqlx::query(
            r#"
            INSERT INTO execution_events (aggregate_id, sequence_number, event_type, timestamp, payload, version)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(event.aggregate_id)
        .bind(event.sequence_number as i64)
        .bind(&event.event_type)
        .bind(event.timestamp)
        .bind(payload_json)
        .bind(event.version as i32)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn load_events(
        &self,
        aggregate_id: uuid::Uuid,
        after_sequence: u64,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        use sqlx::Row;

        let records = sqlx::query(
            r#"
            SELECT aggregate_id, sequence_number, event_type, timestamp, payload, version
            FROM execution_events
            WHERE aggregate_id = $1 AND sequence_number > $2
            ORDER BY sequence_number ASC
            "#
        )
        .bind(aggregate_id)
        .bind(after_sequence as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(records.len());
        for r in records {
            let payload_value: serde_json::Value = r.get("payload");
            let payload = serde_json::from_value(payload_value).unwrap();
            let timestamp_dt: time::OffsetDateTime = r.get("timestamp");

            events.push(EventRecord {
                aggregate_id: r.get("aggregate_id"),
                sequence_number: r.get::<i64, _>("sequence_number") as u64,
                event_type: r.get("event_type"),
                timestamp: timestamp_dt,
                payload,
                version: r.get::<i32, _>("version") as u32,
            });
        }

        Ok(events)
    }

    pub async fn append_snapshot(
        tx: &mut Transaction<'_, Postgres>,
        snapshot: &SnapshotRecord,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO execution_snapshots (aggregate_id, snapshot_version, sequence_number, payload)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(snapshot.aggregate_id)
        .bind(snapshot.snapshot_version as i32)
        .bind(snapshot.sequence_number as i64)
        .bind(&snapshot.payload)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn load_latest_snapshot(
        &self,
        aggregate_id: uuid::Uuid,
    ) -> Result<Option<SnapshotRecord>, sqlx::Error> {
        use sqlx::Row;
        
        let record = sqlx::query(
            r#"
            SELECT aggregate_id, snapshot_version, sequence_number, payload
            FROM execution_snapshots
            WHERE aggregate_id = $1
            ORDER BY sequence_number DESC
            LIMIT 1
            "#
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| {
            let payload: serde_json::Value = r.get("payload");
            SnapshotRecord {
                aggregate_id: r.get("aggregate_id"),
                snapshot_version: r.get::<i32, _>("snapshot_version") as u32,
                sequence_number: r.get::<i64, _>("sequence_number") as u64,
                payload,
            }
        }))
    }
}
