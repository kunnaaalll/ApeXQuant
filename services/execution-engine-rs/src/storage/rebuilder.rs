use crate::storage::aggregate::Aggregatable;
use crate::storage::events::EventRecord;
use crate::storage::snapshots::SnapshotRecord;
use crate::storage::sequence::validate_sequence_strict;
use crate::storage::StorageError;

pub struct ExecutionEventRebuilder;

impl ExecutionEventRebuilder {
    pub fn rebuild<A: Aggregatable + Default>(
        snapshot_opt: Option<SnapshotRecord>,
        events: Vec<EventRecord>,
    ) -> Result<A, StorageError> {
        let mut aggregate = A::default();
        let mut current_sequence = 0;

        if let Some(snapshot) = snapshot_opt {
            aggregate.restore_snapshot(snapshot.payload);
            current_sequence = snapshot.sequence_number;
        }

        for event in events {
            validate_sequence_strict(current_sequence, event.sequence_number)?;
            aggregate.apply_event(&event.payload);
            current_sequence = event.sequence_number;
        }

        Ok(aggregate)
    }
}
