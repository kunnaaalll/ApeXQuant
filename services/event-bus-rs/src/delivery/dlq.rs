use apex_protos::events::{Event, DeadLetterEntry};
use anyhow::Result;

pub struct DeadLetterQueueManager {
    // pool: sqlx::PgPool
}

impl DeadLetterQueueManager {
    pub async fn move_to_dlq(&self, entry: DeadLetterEntry, original_payload: Vec<u8>) -> Result<()> {
        // sqlx insertion into dead_letter_queue table
        Ok(())
    }

    pub async fn replay_from_dlq(&self, id: uuid::Uuid) -> Result<()> {
        // read from dead_letter_queue and re-inject into event bus
        Ok(())
    }
}
