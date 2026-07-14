use apex_protos::events::Event;
use apex_protos::common::Timestamp;
use anyhow::{Result, Context};
use crate::redis::RedisManager;

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
        if topic_str.starts_with("market.") || 
           topic_str.starts_with("strategy.") || 
           topic_str.starts_with("risk.") || 
           topic_str.starts_with("execution.") || 
           topic_str.starts_with("portfolio.") || 
           topic_str.starts_with("system.") {
            Ok(topic_str.to_string())
        } else {
            Ok(topic_str.to_string())
        }
    }
}

#[derive(Clone)]
pub struct SequenceManager {
    redis: RedisManager,
}

impl SequenceManager {
    pub fn new(redis: RedisManager) -> Self {
        Self { redis }
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
