use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecommendationEvent {
    pub model_id: String,
    pub symbol: String,
    pub target_weight: Decimal,
    pub conviction: Decimal,
    pub timestamp: i64,
}

pub struct LearningClient;

impl Default for LearningClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningClient {
    pub fn new() -> Self {
        Self
    }
}
