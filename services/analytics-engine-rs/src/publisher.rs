//! Event Bus Publisher — publish analytics results to Event Bus.

use serde::Serialize;

/// Serialise and publish a metrics snapshot to the event bus topic.
pub async fn publish_metrics_snapshot<T: Serialize>(
    redis_client: &redis::Client,
    topic: &str,
    payload: &T,
) -> Result<(), String> {
    let json =
        serde_json::to_string(payload).map_err(|e| format!("Serialisation failed: {}", e))?;

    let mut conn = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| format!("Redis connection failed: {}", e))?;

    redis::cmd("PUBLISH")
        .arg(topic)
        .arg(&json)
        .query_async::<_, i64>(&mut conn)
        .await
        .map_err(|e| format!("Redis PUBLISH failed: {}", e))?;

    tracing::debug!("Published {} bytes to topic '{}'", json.len(), topic);
    Ok(())
}
