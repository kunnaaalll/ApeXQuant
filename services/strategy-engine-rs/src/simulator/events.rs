use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationCompletedEvent {
    pub run_id: String,
    pub timestamp: u64,
}
