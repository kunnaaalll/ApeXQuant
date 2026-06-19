use super::events::EventRecord;
use super::snapshots::SnapshotRecord;
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Debug)]
pub enum StoreError {
    DatabaseError(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::DatabaseError(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        StoreError::DatabaseError(err.to_string())
    }
}

pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append_event(&self, event: &EventRecord) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        self.append_event_tx(&mut tx, event).await?;
        tx.commit().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn append_event_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &EventRecord,
    ) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            INSERT INTO strategy_events (event_id, aggregate_id, sequence, timestamp, event_type, payload)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.aggregate_id)
        .bind(event.sequence)
        .bind(event.timestamp)
        .bind(&event.event_type)
        .bind(&event.payload)
        .execute(&mut **tx)
        .await
        .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn append_events(&self, events: &[EventRecord]) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        for event in events {
            self.append_event_tx(&mut tx, event).await?;
        }

        tx.commit().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn append_events_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        events: &[EventRecord],
    ) -> Result<(), StoreError> {
        for event in events {
            self.append_event_tx(tx, event).await?;
        }
        Ok(())
    }

    pub async fn load_events(
        &self,
        aggregate_id: &str,
        from_sequence: i64,
    ) -> Result<Vec<EventRecord>, StoreError> {
        let records = sqlx::query_as::<_, (String, String, i64, i64, String, serde_json::Value)>(
            r#"
            SELECT event_id, aggregate_id, sequence, timestamp, event_type, payload
            FROM strategy_events
            WHERE aggregate_id = $1 AND sequence > $2
            ORDER BY sequence ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(from_sequence)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        let events = records
            .into_iter()
            .map(|(event_id, aggregate_id, sequence, timestamp, event_type, payload)| EventRecord {
                event_id,
                aggregate_id,
                sequence,
                timestamp,
                event_type,
                payload,
            })
            .collect();

        Ok(events)
    }

    pub async fn save_snapshot(&self, snapshot: &SnapshotRecord) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        self.save_snapshot_tx(&mut tx, snapshot).await?;
        tx.commit().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn save_snapshot_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        snapshot: &SnapshotRecord,
    ) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            INSERT INTO strategy_snapshots (aggregate_id, sequence, timestamp, snapshot_payload)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (aggregate_id, sequence) DO NOTHING
            "#,
        )
        .bind(&snapshot.aggregate_id)
        .bind(snapshot.sequence)
        .bind(snapshot.timestamp)
        .bind(&snapshot.snapshot_payload)
        .execute(&mut **tx)
        .await
        .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<SnapshotRecord>, StoreError> {
        let record = sqlx::query_as::<_, (String, i64, i64, serde_json::Value)>(
            r#"
            SELECT aggregate_id, sequence, timestamp, snapshot_payload
            FROM strategy_snapshots
            WHERE aggregate_id = $1
            ORDER BY sequence DESC
            LIMIT 1
            "#,
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        if let Some((aggregate_id, sequence, timestamp, snapshot_payload)) = record {
            Ok(Some(SnapshotRecord {
                aggregate_id,
                sequence,
                timestamp,
                snapshot_payload,
            }))
        } else {
            Ok(None)
        }
    }
}
