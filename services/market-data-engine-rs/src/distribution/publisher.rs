use super::types::{EventEnvelope, MarketTopic};
use tokio::sync::broadcast;
use std::collections::HashMap;

pub struct MarketEventPublisher<T> {
    channels: HashMap<MarketTopic, broadcast::Sender<EventEnvelope<T>>>,
}

impl<T: Clone + Send + 'static> MarketEventPublisher<T> {
    pub fn new(capacity: usize) -> Self {
        let mut channels = HashMap::new();
        let topics = vec![
            MarketTopic::TickEvents,
            MarketTopic::CandleEvents,
            MarketTopic::VolatilityEvents,
            MarketTopic::RegimeEvents,
            MarketTopic::CorrelationEvents,
            MarketTopic::IntelligenceEvents,
            MarketTopic::QualityEvents,
            MarketTopic::SessionEvents,
        ];
        for topic in topics {
            let (tx, _) = broadcast::channel(capacity);
            channels.insert(topic, tx);
        }
        Self { channels }
    }

    pub fn publish(&self, envelope: EventEnvelope<T>) -> Result<usize, String> {
        if let Some(tx) = self.channels.get(&envelope.topic) {
            tx.send(envelope).map_err(|e| format!("Failed to publish: {}", e))
        } else {
            Err(format!("Topic not configured: {:?}", envelope.topic))
        }
    }
    
    pub fn get_channels(&self) -> &HashMap<MarketTopic, broadcast::Sender<EventEnvelope<T>>> {
        &self.channels
    }
}
