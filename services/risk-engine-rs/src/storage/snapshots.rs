use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub aggregate_id: Uuid,
    pub version: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub snapshot: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapshotFrequency {
    Every10Events,
    Every50Events,
    Every100Events,
    Manual,
}

impl SnapshotFrequency {
    pub fn should_snapshot(&self, sequence: i64) -> bool {
        match self {
            Self::Every10Events => sequence > 0 && sequence % 10 == 0,
            Self::Every50Events => sequence > 0 && sequence % 50 == 0,
            Self::Every100Events => sequence > 0 && sequence % 100 == 0,
            Self::Manual => false,
        }
    }
}
