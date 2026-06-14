use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeMetrics {
    pub raw_edge: Decimal,
    pub edge_score: Decimal,
    pub edge_acceleration: Decimal,
    pub edge_decay: Decimal,
    pub edge_confidence: Decimal,
    pub edge_stability: Decimal,
}

impl Default for EdgeMetrics {
    fn default() -> Self {
        Self {
            raw_edge: Decimal::ZERO,
            edge_score: Decimal::ZERO,
            edge_acceleration: Decimal::ZERO,
            edge_decay: Decimal::ZERO,
            edge_confidence: Decimal::ZERO,
            edge_stability: Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeAssessment {
    pub metrics: EdgeMetrics,
    pub state: super::states::EdgeState,
}
