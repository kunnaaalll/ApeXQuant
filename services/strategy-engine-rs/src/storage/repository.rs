use super::events::{EventRecord, StrategyEventWrapper};
use super::pg_store::{PgStore, StoreError};
use super::rebuilder::{Aggregatable, StrategyEventRebuilder};
use super::serializer::Serializer;
use super::snapshots::{SnapshotFrequency, SnapshotRecord};
use sqlx::PgPool;

#[derive(Debug)]
pub enum RepositoryError {
    Database(StoreError),
    Serialization(String),
    Rebuild(String),
    TransactionFailed(String),
}

impl From<StoreError> for RepositoryError {
    fn from(err: StoreError) -> Self {
        RepositoryError::Database(err)
    }
}

pub struct StrategyRepository {
    store: PgStore,
    pool: PgPool,
}

impl StrategyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            store: PgStore::new(pool.clone()),
            pool,
        }
    }

    /// Load an aggregate by replaying its events. Uses snapshot if available.
    pub async fn load_aggregate<A: Aggregatable>(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<A>, RepositoryError> {
        let snapshot_record = self.store.load_snapshot(aggregate_id).await?;
        
        let (from_sequence, snapshot) = match snapshot_record {
            Some(record) => {
                let snap = Serializer::deserialize::<A::Snapshot>(record.snapshot_payload)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                (record.sequence, Some(snap))
            }
            None => (0, None),
        };

        let event_records = self.store.load_events(aggregate_id, from_sequence).await?;
        
        if snapshot.is_none() && event_records.is_empty() {
            return Ok(None);
        }

        let mut wrappers = Vec::with_capacity(event_records.len());
        for record in event_records {
            let wrapper = Serializer::deserialize::<StrategyEventWrapper>(record.payload)
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            wrappers.push(wrapper);
        }

        let aggregate = StrategyEventRebuilder::rebuild::<A>(snapshot, &wrappers)
            .map_err(|_| RepositoryError::Rebuild("Failed to rebuild aggregate".to_string()))?;

        Ok(Some(aggregate))
    }

    /// Save new events for an aggregate. Automatically manages snapshots based on frequency.
    pub async fn save_events<A: Aggregatable>(
        &self,
        aggregate_id: &str,
        events: &[(StrategyEventWrapper, i64, i64)], // (event, sequence, timestamp)
        current_aggregate: &A,
        snapshot_frequency: SnapshotFrequency,
        last_snapshot_sequence: i64,
    ) -> Result<(), RepositoryError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut event_records = Vec::with_capacity(events.len());
        for (event_wrapper, sequence, timestamp) in events {
            let payload = Serializer::serialize(event_wrapper)
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            event_records.push(EventRecord {
                event_id: uuid::Uuid::new_v4().to_string(),
                aggregate_id: aggregate_id.to_string(),
                sequence: *sequence,
                timestamp: *timestamp,
                event_type: "StrategyEventWrapper".to_string(),
                payload,
            });
        }

        let latest_sequence = events.last().map(|(_, seq, _)| *seq).unwrap_or(0);
        let latest_timestamp = events.last().map(|(_, _, ts)| *ts).unwrap_or(0);

        let mut tx = self.pool.begin().await
            .map_err(|e| RepositoryError::TransactionFailed(e.to_string()))?;

        self.store.append_events_tx(&mut tx, &event_records).await?;

        if snapshot_frequency.should_snapshot(latest_sequence, last_snapshot_sequence) {
            let snapshot_payload = Serializer::serialize(&current_aggregate.snapshot())
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            let snapshot_record = SnapshotRecord {
                aggregate_id: aggregate_id.to_string(),
                sequence: latest_sequence,
                timestamp: latest_timestamp,
                snapshot_payload,
            };

            self.store.save_snapshot_tx(&mut tx, &snapshot_record).await?;
        }

        tx.commit().await
            .map_err(|e| RepositoryError::TransactionFailed(e.to_string()))?;

        Ok(())
    }
}
