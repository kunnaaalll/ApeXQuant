use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDistributionEvent {
    pub id: String,
    pub topic: String,
    pub payload: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSnapshot {
    pub id: String,
    pub active_subscriptions: usize,
    pub events_published: u64,
    pub timestamp: DateTime<Utc>,
}
