//! Event publisher implementation

use crate::{Event, EventBusError, Result};
use redis::aio::ConnectionManager;
use tracing::{debug, trace, warn};

/// Event publisher for Redis Streams
#[derive(Clone)]
pub struct EventPublisher {
    redis: ConnectionManager,
}

impl EventPublisher {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    /// Publish a single event
    pub async fn publish(&self, event: Event) -> Result<String> {
        let stream_key = event.stream_key();
        let fields = event.to_stream_fields()?;

        trace!(
            event_id = %event.id,
            topic = %event.metadata.topic,
            event_type = %event.metadata.event_type,
            "Publishing event"
        );

        let id: String = redis::Cmd::xadd(
            &stream_key,
            "*", // Auto-generate ID
            &fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>(),
        )
        .query_async(&mut self.redis.clone())
        .await
        .map_err(|e| EventBusError::Publish(e.to_string()))?;

        debug!(
            event_id = %event.id,
            stream_id = %id,
            topic = %event.metadata.topic,
            "Event published successfully"
        );

        Ok(id)
    }

    /// Publish multiple events (batch)
    pub async fn publish_batch(&self, events: Vec<Event>) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(events.len());

        // Group events by topic for potential pipelining
        let mut pipeline = redis::Pipeline::new();
        let mut stream_keys = Vec::new();

        for event in &events {
            let stream_key = event.stream_key();
            let fields = event.to_stream_fields()?;

            pipeline.xadd(
                &stream_key,
                "*",
                &fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>(),
            );
            stream_keys.push(stream_key);
        }

        let results: Vec<String> = pipeline
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| EventBusError::Publish(e.to_string()))?;

        for (i, id) in results.iter().enumerate() {
            if let Some(event) = events.get(i) {
                trace!(
                    event_id = %event.id,
                    stream_id = %id,
                    topic = %event.metadata.topic,
                    "Batch event published"
                );
            }
        }

        ids.extend(results);

        debug!(count = events.len(), "Batch publish completed");

        Ok(ids)
    }

    /// Publish with maxlen to cap stream size
    pub async fn publish_with_limit(&self, event: Event, maxlen: usize) -> Result<String> {
        let stream_key = event.stream_key();
        let fields = event.to_stream_fields()?;

        // Use XADD with MAXLEN ~ for approximate trimming
        let id: String = redis::Cmd::xadd_maxlen(
            &stream_key,
            maxlen,
            "*",
            &fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>(),
        )
        .query_async(&mut self.redis.clone())
        .await
        .map_err(|e| EventBusError::Publish(e.to_string()))?;

        Ok(id)
    }
}
