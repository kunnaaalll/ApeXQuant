use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSnapshot {
    pub strategy_id: String,
    pub active_generation: u32,
    pub timestamp: u64,
}
