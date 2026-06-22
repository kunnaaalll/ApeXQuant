use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub aggregate_id: Uuid,
    pub snapshot_version: u32,
    pub sequence_number: u64,
    pub payload: serde_json::Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotFrequency {
    Every10Events,
    Every50Events,
    Every100Events,
    Every500Events,
    Every1000Events,
}

impl SnapshotFrequency {
    pub fn threshold(&self) -> u64 {
        match self {
            SnapshotFrequency::Every10Events => 10,
            SnapshotFrequency::Every50Events => 50,
            SnapshotFrequency::Every100Events => 100,
            SnapshotFrequency::Every500Events => 500,
            SnapshotFrequency::Every1000Events => 1000,
        }
    }

    pub fn should_snapshot(&self, current_sequence: u64) -> bool {
        current_sequence > 0 && current_sequence % self.threshold() == 0
    }
}
