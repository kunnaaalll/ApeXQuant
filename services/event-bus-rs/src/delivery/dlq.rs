use apex_protos::events::DeadLetterEntry;
use anyhow::Result;
use crate::storage::db::EventStore;
use std::sync::Arc;

pub struct DeadLetterQueueManager {
    store: Arc<EventStore>,
}

impl DeadLetterQueueManager {
    pub fn new(store: Arc<EventStore>) -> Self {
        Self { store }
    }

    pub async fn move_to_dlq(
        &self,
        consumer_group: &str,
        topic: &str,
        entry: DeadLetterEntry,
        original_payload: Vec<u8>
    ) -> Result<()> {
        let event_id = if entry.event_id.is_empty() {
            None
        } else {
            uuid::Uuid::parse_str(&entry.event_id).ok()
        };
        
        let error_details = entry.error.as_ref().map(|e| e.message.clone()).unwrap_or_default();
        
        self.store.move_to_dlq(
            consumer_group,
            topic,
            event_id,
            &original_payload,
            &entry.reason,
            &error_details,
        ).await?;
        
        Ok(())
    }

    pub async fn replay_from_dlq(&self, id: uuid::Uuid) -> Result<()> {
        let entry = self.store.fetch_from_dlq(id).await?;
        if let Some((payload, _topic)) = entry {
            use prost::Message;
            if let Ok(event) = apex_protos::events::Event::decode(&payload[..]) {
                self.store.store_event(&event).await?;
                tracing::info!("DLQ entry {} replayed successfully", id);
            } else {
                tracing::warn!("DLQ entry {} payload could not be decoded", id);
            }
        } else {
            tracing::warn!("DLQ entry {} not found", id);
        }
        Ok(())
    }
}
