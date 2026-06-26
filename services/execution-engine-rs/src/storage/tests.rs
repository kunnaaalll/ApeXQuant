use uuid::Uuid;
use time::OffsetDateTime;
use serde_json::json;

use crate::storage::events::{EventRecord, ExecutionEventWrapper};
use crate::storage::sequence::validate_sequence_strict;
use crate::storage::versioning::AggregateVersion;
use crate::storage::StorageError;
use crate::storage::aggregate::Aggregatable;
use crate::storage::rebuilder::ExecutionEventRebuilder;

#[derive(Default, Debug, PartialEq)]
struct ExecutionTestAggregate {
    count: u64,
}

impl Aggregatable for ExecutionTestAggregate {
    fn apply_event(&mut self, _event: &ExecutionEventWrapper) {
        self.count += 1;
    }

    fn snapshot(&self) -> serde_json::Value {
        json!({ "count": self.count })
    }

    fn restore_snapshot(&mut self, payload: serde_json::Value) {
        self.count = payload["count"].as_u64().unwrap_or(0);
    }
}

fn create_test_event(seq: u64) -> EventRecord {
    EventRecord {
        aggregate_id: Uuid::new_v4(),
        sequence_number: seq,
        event_type: "TestEvent".to_string(),
        timestamp: OffsetDateTime::now_utc(),
        payload: ExecutionEventWrapper::OrderEvent(json!({})),
        version: 1,
    }
}

#[test]
fn test_event_ordering() {
    assert!(validate_sequence_strict(1, 2).is_ok());
    assert!(validate_sequence_strict(2, 3).is_ok());
    assert!(validate_sequence_strict(3, 4).is_ok());
}

#[test]
fn test_sequence_violation() {
    let result = validate_sequence_strict(1, 3);
    assert!(matches!(result, Err(StorageError::SequenceViolation { .. })));

    let result = validate_sequence_strict(5, 2);
    assert!(matches!(result, Err(StorageError::SequenceViolation { .. })));
}

#[test]
fn test_snapshot_restore() {
    let mut aggregate = ExecutionTestAggregate::default();
    aggregate.apply_event(&ExecutionEventWrapper::OrderEvent(json!({})));
    aggregate.apply_event(&ExecutionEventWrapper::OrderEvent(json!({})));
    
    let snap = aggregate.snapshot();
    
    let mut new_aggregate = ExecutionTestAggregate::default();
    new_aggregate.restore_snapshot(snap);
    
    assert_eq!(aggregate, new_aggregate);
}

#[test]
fn test_event_rebuild() {
    let mut aggregate = ExecutionTestAggregate::default();
    let events = vec![
        create_test_event(1),
        create_test_event(2),
        create_test_event(3),
    ];
    
    for e in &events {
        aggregate.apply_event(&e.payload);
    }
    
    let rebuilt = ExecutionEventRebuilder::rebuild::<ExecutionTestAggregate>(None, events).unwrap();
    assert_eq!(aggregate, rebuilt);
}

#[tokio::test]
#[ignore]
async fn test_repository_roundtrip() {
    // Requires DB connection
}

#[test]
fn test_version_monotonicity() {
    let v1 = AggregateVersion(1);
    let v2 = AggregateVersion(2);
    let v7 = AggregateVersion(7);
    let v6 = AggregateVersion(6);

    assert!(AggregateVersion::validate_transition(v1, v2).is_ok());
    assert!(AggregateVersion::validate_transition(v7, v6).is_err());
    assert!(AggregateVersion::validate_transition(v1, v1).is_err());
}

#[tokio::test]
#[ignore]
async fn test_transaction_atomicity() {
    // Requires DB connection
}

#[test]
fn test_determinism_100k_iterations() {
    let mut aggregate = ExecutionTestAggregate::default();
    for i in 1..=100_000 {
        let event = create_test_event(i);
        aggregate.apply_event(&event.payload);
    }
    assert_eq!(aggregate.count, 100_000);
}

#[tokio::test]
#[ignore]
async fn test_append_event() {
    // Integration
}

#[tokio::test]
#[ignore]
async fn test_load_events() {
    // Integration
}

#[tokio::test]
#[ignore]
async fn test_snapshot_storage() {
    // Integration
}

#[tokio::test]
#[ignore]
async fn test_concurrent_append() {
    // Integration
}
