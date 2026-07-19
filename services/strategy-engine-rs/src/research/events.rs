use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRankedEvent {
    pub edge_id: String,
    pub rank_score: Decimal,
    pub timestamp: u64,
}
