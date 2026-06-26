use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAllocationRecommendationEvent {
    pub ai_model_id: String,
    pub target_allocation: Decimal,
    pub risk_adjustment: Decimal,
    pub timestamp: i64,
}

pub struct AiClient;

impl Default for AiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AiClient {
    pub fn new() -> Self {
        Self
    }
}
