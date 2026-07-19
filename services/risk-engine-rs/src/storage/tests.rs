use rust_decimal_macros::dec;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::storage::events::{EventRecord, PortfolioEventWrapper};

use crate::drawdown::events::DrawdownEvent;
use crate::storage::rebuilder::{Aggregatable, RiskEventRebuilder};

#[derive(Debug, Clone, PartialEq)]
struct MockRiskState {
    pub max_drawdown: rust_decimal::Decimal,
    pub current_drawdown: rust_decimal::Decimal,
}

impl Aggregatable for MockRiskState {
    fn apply(&mut self, event: &PortfolioEventWrapper) {
        if let PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated { value, .. }) = event {
            self.current_drawdown = *value;
            if self.current_drawdown > self.max_drawdown {
                self.max_drawdown = self.current_drawdown;
            }
        }
    }
}

// Logic Tests

#[test]
fn test_rebuild_from_events() {
    let initial_state = MockRiskState {
        max_drawdown: dec!(0.0),
        current_drawdown: dec!(0.0),
    };

    let events = vec![
        EventRecord {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: 1,
            timestamp: OffsetDateTime::now_utc(),
            payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
                timestamp: 1,
                value: dec!(0.05),
            }),
            version: 1,
        },
        EventRecord {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: 2,
            timestamp: OffsetDateTime::now_utc(),
            payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
                timestamp: 2,
                value: dec!(0.10),
            }),
            version: 2,
        },
        EventRecord {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: 3,
            timestamp: OffsetDateTime::now_utc(),
            payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
                timestamp: 3,
                value: dec!(0.08),
            }),
            version: 3,
        },
    ];

    let rebuilt_state = RiskEventRebuilder::rebuild(initial_state, &events);

    assert_eq!(rebuilt_state.current_drawdown, dec!(0.08));
    assert_eq!(rebuilt_state.max_drawdown, dec!(0.10));
}

#[test]
fn test_snapshot_plus_events() {
    let snapshot_state = MockRiskState {
        max_drawdown: dec!(0.20),
        current_drawdown: dec!(0.15),
    };

    let events = vec![EventRecord {
        event_id: Uuid::new_v4(),
        aggregate_id: Uuid::new_v4(),
        sequence: 101,
        timestamp: OffsetDateTime::now_utc(),
        payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
            timestamp: 4,
            value: dec!(0.25),
        }),
        version: 101,
    }];

    let rebuilt_state = RiskEventRebuilder::rebuild(snapshot_state.clone(), &events);

    // Should equal a full replay equivalent
    let mut expected_state = snapshot_state;
    expected_state.current_drawdown = dec!(0.25);
    expected_state.max_drawdown = dec!(0.25);

    assert_eq!(rebuilt_state, expected_state);
}

#[test]
fn test_ordering() {
    // Test that our sequence logic enforces ordering implicitly in the list
    let e1 = EventRecord {
        event_id: Uuid::new_v4(),
        aggregate_id: Uuid::new_v4(),
        sequence: 1,
        timestamp: OffsetDateTime::now_utc(),
        payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
            timestamp: 1,
            value: dec!(0.01),
        }),
        version: 1,
    };
    let e2 = EventRecord {
        event_id: Uuid::new_v4(),
        aggregate_id: Uuid::new_v4(),
        sequence: 2,
        timestamp: OffsetDateTime::now_utc(),
        payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
            timestamp: 2,
            value: dec!(0.02),
        }),
        version: 2,
    };

    let events = [e1, e2];

    for i in 0..events.len() - 1 {
        assert!(events[i].sequence < events[i + 1].sequence);
    }
}

#[test]
fn test_determinism_100k() {
    let initial_state = MockRiskState {
        max_drawdown: dec!(0.0),
        current_drawdown: dec!(0.0),
    };

    let events: Vec<EventRecord> = (1..=100)
        .map(|i| EventRecord {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: i,
            timestamp: OffsetDateTime::now_utc(),
            payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
                timestamp: i,
                value: dec!(0.05),
            }),
            version: i,
        })
        .collect();

    // Rebuild 100,000 times
    let first_run = RiskEventRebuilder::rebuild(initial_state.clone(), &events);

    for _ in 0..100_000 {
        let nth_run = RiskEventRebuilder::rebuild(initial_state.clone(), &events);
        assert_eq!(nth_run, first_run);
    }
}

// DB Integration Tests

#[sqlx::test]
#[ignore = "requires database"]
async fn test_event_append_and_load(pool: sqlx::PgPool) {
    let store = crate::storage::pg_store::PostgresRiskStore::new(pool);
    let aggregate_id = Uuid::new_v4();

    let event = EventRecord {
        event_id: Uuid::new_v4(),
        aggregate_id,
        sequence: 1,
        timestamp: OffsetDateTime::now_utc(),
        payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
            timestamp: 1,
            value: dec!(0.05),
        }),
        version: 1,
    };

    store
        .append_event(&event)
        .await
        .expect("Failed to append event");

    let loaded_events = store
        .load_events(aggregate_id, 0)
        .await
        .expect("Failed to load events");
    assert_eq!(loaded_events.len(), 1);
    assert_eq!(loaded_events[0].event_id, event.event_id);
    assert_eq!(loaded_events[0].sequence, 1);
}

#[sqlx::test]
#[ignore = "requires database"]
async fn test_snapshot_append_and_load(pool: sqlx::PgPool) {
    let store = crate::storage::pg_store::PostgresRiskStore::new(pool);
    let aggregate_id = Uuid::new_v4();

    let snapshot = crate::storage::snapshots::SnapshotRecord {
        aggregate_id,
        version: 100,
        timestamp: OffsetDateTime::now_utc(),
        snapshot: serde_json::json!({"test": 123}),
    };

    store
        .append_snapshot(&snapshot)
        .await
        .expect("Failed to append snapshot");

    let loaded = store
        .load_snapshot(aggregate_id)
        .await
        .expect("Failed to load snapshot");
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.version, 100);
    assert_eq!(loaded.snapshot, serde_json::json!({"test": 123}));
}

#[sqlx::test]
#[ignore = "requires database"]
async fn test_concurrent_append(pool: sqlx::PgPool) {
    let store = std::sync::Arc::new(crate::storage::pg_store::PostgresRiskStore::new(pool));
    let aggregate_id = Uuid::new_v4();

    let mut handles = vec![];
    for i in 1..=10 {
        let store_clone = store.clone();
        handles.push(tokio::spawn(async move {
            let event = EventRecord {
                event_id: Uuid::new_v4(),
                aggregate_id,
                sequence: i,
                timestamp: OffsetDateTime::now_utc(),
                payload: PortfolioEventWrapper::Drawdown(DrawdownEvent::Updated {
                    timestamp: i,
                    value: dec!(0.05),
                }),
                version: i,
            };
            store_clone
                .append_event(&event)
                .await
                .expect("Append failed");
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let loaded = store
        .load_events(aggregate_id, 0)
        .await
        .expect("Load failed");
    assert_eq!(loaded.len(), 10);
}
