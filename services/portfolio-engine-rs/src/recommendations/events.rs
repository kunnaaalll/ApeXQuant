use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationEvent {
    HeatChanged { old_score: u8, new_score: u8 },
    HealthChanged { old_score: u8, new_score: u8 },
    QualityChanged { old_score: u8, new_score: u8 },
    DrawdownChanged { is_critical: bool },
    CorrelationChanged,
    AllocationChanged,
}
