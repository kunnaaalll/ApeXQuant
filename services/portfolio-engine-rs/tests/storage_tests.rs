#![allow(warnings, clippy::all, deprecated)]
use anyhow::Result;
use portfolio_engine::storage::{
    EventRebuilder, EventRecord, PortfolioEventWrapper, PortfolioRepository,
    PortfolioSnapshotWrapper, PostgresPortfolioStore, SnapshotFrequency, SnapshotRecord,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

// Note: These tests require a running PostgreSQL instance and a proper schema.
// They are marked with #[ignore] so they don't break CI without a database.
// To run them: `cargo test -- --ignored` or set up the test DB in the pipeline.

async fn setup_db() -> Result<PgPool> {
    // In a real scenario, this would use a test DB URL from the environment
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/apex_test").await?;

    // We would typically run migrations here
    // sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[tokio::test]
#[ignore]
async fn test_event_append_and_load() -> Result<()> {
    let pool = setup_db().await?;
    let store = PostgresPortfolioStore::new(pool);
    let repo = PortfolioRepository::new(store);

    let aggregate_id = Uuid::new_v4().to_string();
    let payload = PortfolioEventWrapper::Portfolio(json!({"action": "create"}));

    let event = EventRecord::new(
        &aggregate_id,
        1,
        "PortfolioCreated",
        payload.clone(),
        json!({"source": "test"}),
    );

    repo.save_event_with_snapshot(&event, None).await?;

    let events = repo.load_events(&aggregate_id).await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].version, 1);
    assert_eq!(events[0].event_type, "PortfolioCreated");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_snapshot_append_and_load() -> Result<()> {
    let pool = setup_db().await?;
    let store = PostgresPortfolioStore::new(pool);
    let repo = PortfolioRepository::new(store);

    let aggregate_id = Uuid::new_v4().to_string();
    let payload = PortfolioSnapshotWrapper::Portfolio(json!({"state": "active"}));

    let snapshot = SnapshotRecord::new(
        &aggregate_id,
        10,
        "PortfolioState",
        SnapshotFrequency::Realtime,
        payload,
    );

    repo.save_snapshot(&snapshot).await?;

    let loaded = repo
        .load_latest_snapshot(&aggregate_id, SnapshotFrequency::Realtime)
        .await?;
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.version, 10);
    assert_eq!(loaded.frequency, SnapshotFrequency::Realtime);

    Ok(())
}

#[test]
fn test_event_rebuilder_logic() -> Result<()> {
    // A simple test to ensure EventRebuilder applies functions sequentially
    let initial_state: i32 = 0;

    let events = vec![
        EventRecord::new(
            "agg1",
            1,
            "Add",
            PortfolioEventWrapper::Portfolio(json!(5)),
            json!({}),
        ),
        EventRecord::new(
            "agg1",
            2,
            "Add",
            PortfolioEventWrapper::Portfolio(json!(10)),
            json!({}),
        ),
        EventRecord::new(
            "agg1",
            3,
            "Sub",
            PortfolioEventWrapper::Portfolio(json!(3)),
            json!({}),
        ),
    ];

    let apply_fn = |state: i32, event: &EventRecord| -> Result<i32> {
        let val = if let PortfolioEventWrapper::Portfolio(serde_json::Value::Number(n)) =
            &event.payload
        {
            n.as_i64().unwrap_or(0) as i32
        } else {
            0
        };

        match event.event_type.as_str() {
            "Add" => Ok(state + val),
            "Sub" => Ok(state - val),
            _ => Ok(state),
        }
    };

    let final_state = EventRebuilder::rebuild(initial_state, &events, apply_fn)?;
    assert_eq!(final_state, 12); // 0 + 5 + 10 - 3 = 12

    Ok(())
}
