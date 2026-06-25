use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayWindow {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCheckpoint {
    pub sequence_number: u64,
    pub timestamp: DateTime<Utc>,
    pub state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySnapshot {
    pub checkpoint: ReplayCheckpoint,
    pub data_blob: Vec<u8>,
}
