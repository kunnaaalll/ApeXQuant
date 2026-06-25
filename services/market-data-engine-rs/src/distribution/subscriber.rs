use super::types::{EventEnvelope, MarketTopic};
use tokio::sync::broadcast;
use std::collections::HashMap;

pub struct MarketEventSubscriber<T> {
    receivers: HashMap<MarketTopic, broadcast::Receiver<EventEnvelope<T>>>,
}

impl<T: Clone + Send + 'static> MarketEventSubscriber<T> {
    pub fn new(channels: &HashMap<MarketTopic, broadcast::Sender<EventEnvelope<T>>>, topics: Vec<MarketTopic>) -> Self {
        let mut receivers = HashMap::new();
        for topic in topics {
            if let Some(tx) = channels.get(&topic) {
                receivers.insert(topic, tx.subscribe());
            }
        }
        Self { receivers }
    }

    pub async fn receive(&mut self, topic: MarketTopic) -> Result<EventEnvelope<T>, String> {
        if let Some(rx) = self.receivers.get_mut(&topic) {
            rx.recv().await.map_err(|e| format!("Receive error: {}", e))
        } else {
            Err(format!("Not subscribed to topic: {:?}", topic))
        }
    }
}
