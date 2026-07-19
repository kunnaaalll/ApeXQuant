use crate::redis::RedisManager;
use anyhow::Result;
use apex_protos::common::Timestamp;
use apex_protos::events::Event;

#[derive(Clone)]
pub struct Router {
    pub redis: RedisManager,
}

impl Router {
    pub fn new(redis: RedisManager) -> Self {
        Self { redis }
    }

    pub async fn route_event(&self, event: &Event) -> Result<String> {
        let topic_str = event.topic.as_str();

        // Example deterministic routing logic
        Ok(topic_str.to_string())
    }
}

#[derive(Clone)]
pub struct SequenceManager {
    _redis: RedisManager,
}

impl SequenceManager {
    pub fn new(_redis: RedisManager) -> Self {
        Self { _redis }
    }

    pub async fn sequence_event(&self, mut event: Event) -> Result<Event> {
        // Here we could use Redis for strict distributed sequence assignment
        let now = chrono::Utc::now();
        event.published_at = Some(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        });

        Ok(event)
    }
}
