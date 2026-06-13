//! APEX V3 Event Bus
//!
//! Immutable event streaming infrastructure built on Redis Streams.
//! Provides pub/sub, replay capability, and backpressure handling.

pub mod config;
pub mod error;
pub mod event;
pub mod health;
pub mod metrics;
pub mod publisher;
pub mod server;
pub mod storage;
pub mod subscriber;
pub mod subscription;

pub use config::Config;
pub use error::{EventBusError, Result};
pub use event::{Event, EventMetadata, EventPayload};
pub use health::HealthStatus;

use metrics::EventBusMetrics;
use publisher::EventPublisher;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use storage::EventStorage;
use subscriber::EventSubscriber;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Core event bus handle
#[derive(Clone)]
pub struct EventBus {
    config: Arc<Config>,
    redis: ConnectionManager,
    publisher: EventPublisher,
    storage: Arc<RwLock<EventStorage>>,
    metrics: EventBusMetrics,
}

impl EventBus {
    /// Initialize the event bus with the given configuration
    pub async fn new(config: Config) -> Result<Self> {
        let redis_client = redis::Client::open(config.redis_url.clone())?;
        let redis = ConnectionManager::new(redis_client).await?;

        let publisher = EventPublisher::new(redis.clone());
        let storage = Arc::new(RwLock::new(EventStorage::new(redis.clone(), &config)));
        let metrics = EventBusMetrics::new();

        info!("EventBus initialized with Redis at {}", config.redis_url);

        Ok(Self {
            config: Arc::new(config),
            redis,
            publisher,
            storage,
            metrics,
        })
    }

    /// Publish a single event
    pub async fn publish(&self, event: Event) -> Result<String> {
        let start = std::time::Instant::now();

        // Store in persistent storage
        {
            let storage = self.storage.read().await;
            storage.store(&event).await?;
        }

        // Publish to stream
        let id = self.publisher.publish(event).await?;

        // Record metrics
        self.metrics.record_publish(start.elapsed());

        Ok(id)
    }

    /// Publish multiple events atomically
    pub async fn publish_batch(&self, events: Vec<Event>) -> Result<Vec<String>> {
        let start = std::time::Instant::now();

        // Store all events
        {
            let storage = self.storage.read().await;
            for event in &events {
                storage.store(event).await?;
            }
        }

        // Publish batch
        let ids = self.publisher.publish_batch(events).await?;

        self.metrics.record_batch_publish(ids.len() as u64, start.elapsed());

        Ok(ids)
    }

    /// Create a subscription to a topic
    pub async fn subscribe(&self, topic: &str, group: &str) -> Result<EventSubscriber> {
        let subscriber = EventSubscriber::new(
            self.redis.clone(),
            topic.to_string(),
            group.to_string(),
            self.config.clone(),
        )
        .await?;

        info!("Created subscription to topic='{}' group='{}'", topic, group);

        Ok(subscriber)
    }

    /// Query events from history
    pub async fn query_events(
        &self,
        topic: &str,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
        limit: usize,
    ) -> Result<Vec<Event>> {
        let storage = self.storage.read().await;
        storage.query(topic, from, to, limit).await
    }

    /// Get replay from a specific position
    pub async fn replay_from(
        &self,
        topic: &str,
        stream_id: &str,
        count: usize,
    ) -> Result<Vec<Event>> {
        let storage = self.storage.read().await;
        storage.replay(topic, stream_id, count).await
    }

    /// Trim streams to configured limits
    pub async fn trim_streams(&self) -> Result<u64> {
        let max_len = self.config.stream_max_length;
        let topics: Vec<String> = vec![]; // TODO: Get from config

        let mut total_trimmed = 0u64;

        for topic in topics {
            let stream_key = format!("events:{}", topic);
            let trimmed: i64 = redis::Cmd::xtrim(&stream_key, redis::streams::StreamMaxlen::Approx(max_len))
                .query_async(&mut self.redis.clone())
                .await?;
            total_trimmed += trimmed as u64;
        }

        if total_trimmed > 0 {
            warn!("Trimmed {} old events from streams", total_trimmed);
        }

        Ok(total_trimmed)
    }

    /// Get current health status
    pub async fn health(&self) -> HealthStatus {
        health::check_health(&self.redis).await
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> &EventBusMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event() -> Event {
        Event {
            id: uuid::Uuid::new_v4(),
            metadata: EventMetadata {
                spec_version: "1.0".to_string(),
                occurred_at: chrono::Utc::now(),
                published_at: chrono::Utc::now(),
                event_type: "test.event".to_string(),
                source_service: "test".to_string(),
                topic: "test".to_string(),
                correlation: None,
                causation_id: None,
                deduplication_key: None,
            },
            payload: EventPayload::Raw(serde_json::json!({"test": true})),
        }
    }

    // Integration tests would require Redis to be running
}
