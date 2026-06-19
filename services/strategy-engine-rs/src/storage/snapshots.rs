use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub aggregate_id: String,
    pub sequence: i64,
    pub timestamp: i64,
    pub snapshot_payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotFrequency {
    Every10Events,
    Every50Events,
    Every100Events,
    Every500Events,
}

impl SnapshotFrequency {
    pub fn threshold(&self) -> i64 {
        match self {
            SnapshotFrequency::Every10Events => 10,
            SnapshotFrequency::Every50Events => 50,
            SnapshotFrequency::Every100Events => 100,
            SnapshotFrequency::Every500Events => 500,
        }
    }

    pub fn should_snapshot(&self, current_sequence: i64, last_snapshot_sequence: i64) -> bool {
        current_sequence - last_snapshot_sequence >= self.threshold()
    }
}
