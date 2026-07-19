#![allow(warnings, clippy::all, deprecated)]
//! Integration tests for Event Bus

use common::TestEnvironment;
use std::time::Duration;

mod common;

#[tokio::test]
async fn test_publish_and_subscribe() {
    let env = TestEnvironment::start().await.expect("Failed to start environment");

    let client = env.redis_client().await.expect("Failed to get Redis client");
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect");

    // Test basic publish
    let event_data: Vec<(String, String)> = vec![
        ("event_type".to_string(), "test.event".to_string()),
        ("data".to_string(), "test data".to_string()),
    ];

    let id: String = redis::Cmd::xadd(
        "events:test",
        "*",
        &event_data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>(),
    )
    .query_async(&mut conn)
    .await
    .expect("Failed to publish");

    assert!(!id.is_empty(), "Event ID should not be empty");

    // Test read from stream
    let result: Option<redis::Value> = redis::Cmd::xread(&["events:test"], &["0-0"])
        .query_async(&mut conn)
        .await
        .expect("Failed to read");

    assert!(result.is_some(), "Should have events in stream");
}

#[tokio::test]
async fn test_consumer_group() {
    let env = TestEnvironment::start().await.expect("Failed to start environment");

    let client = env.redis_client().await.expect("Failed to get Redis client");
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect");

    let stream_key = "events:group_test";
    let group_name = "test_consumers";

    // Create consumer group
    let create_result: Result<(), redis::RedisError> = redis::Cmd::xgroup(
        "CREATE",
        stream_key,
        group_name,
        "$",
        "MKSTREAM",
    )
    .query_async(&mut conn)
    .await;

    assert!(create_result.is_ok(), "Should create consumer group");

    // Publish an event
    let id: String = redis::Cmd::xadd(
        stream_key,
        "*",
        &[("test", "value")],
    )
    .query_async(&mut conn)
    .await
    .expect("Failed to publish");

    // Read with consumer group
    let result: Option<redis::Value> = redis::Cmd::xreadgroup(
        "GROUP",
        group_name,
        "consumer1",
        "COUNT",
        10,
        "BLOCK",
        1000,
        "STREAMS",
        stream_key,
        ">",
    )
    .query_async(&mut conn)
    .await
    .expect("Failed to read from group");

    assert!(result.is_some(), "Should receive event from group");
}

#[tokio::test]
async fn test_event_replay() {
    let env = TestEnvironment::start().await.expect("Failed to start environment");

    let client = env.redis_client().await.expect("Failed to get Redis client");
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect");

    let stream_key = "events:replay_test";

    // Publish multiple events
    let mut ids = vec![];
    for i in 0..5 {
        let id: String = redis::Cmd::xadd(
            stream_key,
            "*",
            &[("seq", i.to_string())],
        )
        .query_async(&mut conn)
        .await
        .expect("Failed to publish");
        ids.push(id);
    }

    // Read from second event
    let start_id = &ids[1];
    let result: Option<redis::Value> = redis::Cmd::xrange_count(
        stream_key,
        start_id,
        "+",
        10,
    )
    .query_async(&mut conn)
    .await
    .expect("Failed to range query");

    assert!(result.is_some(), "Should have events for replay");
}

#[tokio::test]
async fn test_stream_trimming() {
    let env = TestEnvironment::start().await.expect("Failed to start environment");

    let client = env.redis_client().await.expect("Failed to get Redis client");
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect");

    let stream_key = "events:trim_test";

    // Add events with maxlen
    for i in 0..10 {
        let _: String = redis::Cmd::xadd_maxlen(
            stream_key,
            5,
            "*",
            &[("seq", i.to_string())],
        )
        .query_async(&mut conn)
        .await
        .expect("Failed to publish");
    }

    // Check stream length
    let len: i64 = redis::Cmd::xlen(stream_key)
        .query_async(&mut conn)
        .await
        .expect("Failed to get length");

    assert!(len <= 6, "Stream should be trimmed to approximately 5 elements, got {}", len);
}

#[tokio::test]
async fn test_concurrent_publishers() {
    let env = TestEnvironment::start().await.expect("Failed to start environment");
    let client = env.redis_client().await.expect("Failed to get Redis client");

    let stream_key = "events:concurrent";
    let num_publishers = 10;
    let events_per_publisher = 100;

    let mut handles = vec![];

    for i in 0..num_publishers {
        let client = client.clone();
        let key = stream_key.to_string();

        let handle = tokio::spawn(async move {
            let mut conn = client.get_multiplexed_async_connection().await?;

            for j in 0..events_per_publisher {
                let _: String = redis::Cmd::xadd(
                    &key,
                    "*",
                    &[("publisher", i.to_string()), ("seq", j.to_string())],
                )
                .query_async(&mut conn)
                .await?;
            }

            Result::<_, redis::RedisError>::Ok(())
        });

        handles.push(handle);
    }

    // Wait for all publishers
    for handle in handles {
        handle.await.expect("Task panicked").expect("Publish failed");
    }

    // Verify total events
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect");
    let len: i64 = redis::Cmd::xlen(stream_key)
        .query_async(&mut conn)
        .await
        .expect("Failed to get length");

    assert_eq!(
        len as usize,
        num_publishers * events_per_publisher,
        "Should have all published events"
    );
}
