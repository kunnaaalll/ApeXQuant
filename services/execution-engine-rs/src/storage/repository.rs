use uuid::Uuid;
use crate::storage::aggregate::Aggregatable;
use crate::storage::events::EventRecord;
use crate::storage::snapshots::{SnapshotRecord, SnapshotFrequency};
use crate::storage::pg_store::PgStore;
use crate::storage::transactions::ExecutionTransaction;
use crate::storage::rebuilder::ExecutionEventRebuilder;
use crate::storage::StorageError;

pub struct ExecutionRepository {
    store: PgStore,
    snapshot_frequency: SnapshotFrequency,
}

impl ExecutionRepository {
    pub fn new(store: PgStore, snapshot_frequency: SnapshotFrequency) -> Self {
        Self {
            store,
            snapshot_frequency,
        }
    }

    pub async fn save<A: Aggregatable>(
        &self,
        aggregate_id: Uuid,
        new_events: &[EventRecord],
        aggregate: &A,
    ) -> Result<(), StorageError> {
        if new_events.is_empty() {
            return Ok(());
        }

        let mut tx = ExecutionTransaction::begin(&self.store).await?;

        let mut last_sequence = 0;
        for event in new_events {
            tx.save_event(event).await?;
            last_sequence = event.sequence_number;
        }

        if self.snapshot_frequency.should_snapshot(last_sequence) {
            let payload = aggregate.snapshot();
            let snapshot = SnapshotRecord {
                aggregate_id,
                snapshot_version: 1, // Will evolve via versioning layer
                sequence_number: last_sequence,
                payload,
            };
            tx.save_snapshot(&snapshot).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load<A: Aggregatable + Default>(
        &self,
        aggregate_id: Uuid,
    ) -> Result<A, StorageError> {
        let snapshot = self.store.load_latest_snapshot(aggregate_id).await?;
        
        let after_sequence = snapshot.as_ref().map(|s| s.sequence_number).unwrap_or(0);
        
        let events = self.store.load_events(aggregate_id, after_sequence).await?;
        
        let aggregate = ExecutionEventRebuilder::rebuild::<A>(snapshot, events)?;
        Ok(aggregate)
    }
}
