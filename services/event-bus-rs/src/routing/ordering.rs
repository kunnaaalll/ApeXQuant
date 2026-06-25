use apex_protos::events::Event;
use apex_protos::common::Timestamp;
use sqlx::PgPool;
use anyhow::{Result, Context};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct SequenceManager {
    pool: PgPool,
    // In a real implementation this would use something like Redis or a strict single-writer actor
    // For now we simulate with an in-memory lock map per topic
    topic_locks: RwLock<HashMap<String, tokio::sync::Mutex<()>>>,
}

impl SequenceManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            topic_locks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn sequence_event(&self, mut event: Event) -> Result<Event> {
        let topic = event.topic.clone();
        
        let mut locks = self.topic_locks.write().await;
        let lock = locks.entry(topic.clone()).or_insert_with(|| tokio::sync::Mutex::new(()));
        
        // This lock ensures strict ordering for insertion within a topic in this node
        let _guard = lock.lock().await;

        // In a fully distributed system, this requires an external sequencer
        // Assign sequence number/strict timestamp here
        let now = chrono::Utc::now();
        event.published_at = Some(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        });

        Ok(event)
    }
}
