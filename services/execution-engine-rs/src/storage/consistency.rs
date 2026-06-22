use uuid::Uuid;
use crate::storage::events::EventRecord;
use crate::storage::snapshots::SnapshotRecord;

#[derive(Debug, PartialEq, Eq)]
pub enum ConsistencyStatus {
    Healthy,
    Warning(String),
    Broken(String),
}

pub struct ConsistencyEngine;

impl ConsistencyEngine {
    pub fn verify_event_count(
        events: &[EventRecord],
        expected_start: u64,
        expected_end: u64,
    ) -> ConsistencyStatus {
        let count = events.len() as u64;
        let expected_count = expected_end.saturating_sub(expected_start);
        
        if count != expected_count {
            ConsistencyStatus::Broken(format!(
                "Event count mismatch: expected {}, got {}",
                expected_count, count
            ))
        } else {
            ConsistencyStatus::Healthy
        }
    }

    pub fn verify_sequence_continuity(events: &[EventRecord]) -> ConsistencyStatus {
        if events.is_empty() {
            return ConsistencyStatus::Healthy;
        }

        let start_seq = events[0].sequence_number;
        for (idx, event) in events.iter().enumerate() {
            let expected_seq = start_seq + idx as u64;
            if event.sequence_number != expected_seq {
                return ConsistencyStatus::Broken(format!(
                    "Sequence broken: expected {}, got {}",
                    expected_seq, event.sequence_number
                ));
            }
        }

        ConsistencyStatus::Healthy
    }

    pub fn verify_snapshot_alignment(
        snapshot: &SnapshotRecord,
        events: &[EventRecord],
    ) -> ConsistencyStatus {
        if let Some(first_event) = events.first() {
            if first_event.sequence_number != snapshot.sequence_number + 1 {
                return ConsistencyStatus::Broken(format!(
                    "Snapshot alignment broken: snapshot seq {}, but next event is {}",
                    snapshot.sequence_number, first_event.sequence_number
                ));
            }
        }
        ConsistencyStatus::Healthy
    }

    pub fn verify_version_consistency(events: &[EventRecord]) -> ConsistencyStatus {
        if events.is_empty() {
            return ConsistencyStatus::Healthy;
        }
        
        let mut last_version = events[0].version;
        for event in events {
            if event.version < last_version {
                return ConsistencyStatus::Broken(format!(
                    "Version regression: {} -> {}",
                    last_version, event.version
                ));
            }
            last_version = event.version;
        }

        ConsistencyStatus::Healthy
    }

    pub fn verify_aggregate_id_consistency(
        aggregate_id: Uuid,
        events: &[EventRecord],
    ) -> ConsistencyStatus {
        for event in events {
            if event.aggregate_id != aggregate_id {
                return ConsistencyStatus::Broken(format!(
                    "Aggregate ID mismatch: expected {}, got {}",
                    aggregate_id, event.aggregate_id
                ));
            }
        }
        ConsistencyStatus::Healthy
    }
}
