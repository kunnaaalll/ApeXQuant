use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum MarketTopic {
    TickEvents,
    CandleEvents,
    VolatilityEvents,
    RegimeEvents,
    CorrelationEvents,
    IntelligenceEvents,
    QualityEvents,
    SessionEvents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryGuarantee {
    AtLeastOnce,
    ExactlyOnce,
    Replayable,
    Ordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub topic: MarketTopic,
    pub priority: EventPriority,
    pub guarantee: DeliveryGuarantee,
    pub payload: T,
    pub sequence_number: u64,
    pub timestamp: DateTime<Utc>,
}
