use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCompletedEvent {
    pub strategy_id: String,
    pub generation: u32,
    pub timestamp: u64,
}
