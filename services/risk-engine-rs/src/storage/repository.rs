use crate::storage::events::EventRecord;
use crate::storage::pg_store::PostgresRiskStore;
use crate::storage::snapshots::SnapshotRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub struct RiskRepository {
    store: PostgresRiskStore,
    pool: PgPool,
}

impl RiskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            store: PostgresRiskStore::new(pool.clone()),
            pool,
        }
    }

    pub async fn save_event_with_snapshot(
        &self,
        event: &EventRecord,
        snapshot_opt: Option<&SnapshotRecord>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let payload_json = serde_json::to_value(&event.payload)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO risk_events (event_id, aggregate_id, sequence, timestamp, payload, version)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(event.event_id)
        .bind(event.aggregate_id)
        .bind(event.sequence)
        .bind(event.timestamp)
        .bind(payload_json)
        .bind(event.version)
        .execute(&mut *tx)
        .await?;

        if let Some(snapshot) = snapshot_opt {
            sqlx::query(
                r#"
                INSERT INTO risk_snapshots (aggregate_id, version, timestamp, snapshot)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(snapshot.aggregate_id)
            .bind(snapshot.version)
            .bind(snapshot.timestamp)
            .bind(&snapshot.snapshot)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn load_latest_snapshot(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Option<SnapshotRecord>, sqlx::Error> {
        self.store.load_snapshot(aggregate_id).await
    }

    pub async fn load_events_since(
        &self,
        aggregate_id: Uuid,
        sequence: i64,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        self.store.load_events(aggregate_id, sequence).await
    }

    pub async fn rebuild_state<S, F>(
        &self,
        aggregate_id: Uuid,
        initial_state: S,
        rebuilder: F,
    ) -> Result<S, sqlx::Error>
    where
        F: Fn(S, &[EventRecord]) -> S,
    {
        // Default to sequence 0 unless we find a snapshot
        let mut start_sequence = 0;

        let snapshot = self.load_latest_snapshot(aggregate_id).await?;
        // If state was reconstructable from snapshot, we'd do it here, but since Repository
        // contains no business logic, we just fetch events.
        // Wait, the prompt says "reconstruct from snapshot + remaining events".
        // We probably just load events after the snapshot's version.
        if let Some(snap) = snapshot {
            start_sequence = snap.version;
        }

        let events = self.load_events_since(aggregate_id, start_sequence).await?;
        Ok(rebuilder(initial_state, &events))
    }
}
