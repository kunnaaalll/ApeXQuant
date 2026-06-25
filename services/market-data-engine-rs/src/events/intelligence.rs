use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceEvent {
    pub symbol: String,
    pub category: String,
    pub payload: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationEvent {
    pub integration_id: String,
    pub payload: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSnapshot {
    pub integration_id: String,
    pub last_event_id: String,
    pub state_blob: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}
