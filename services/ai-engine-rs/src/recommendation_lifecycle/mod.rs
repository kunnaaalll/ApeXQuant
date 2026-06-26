use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationState {
    Generated,
    Shadow,
    Reviewed,
    Approved,
    Production,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendationLifecycleTracker {
    pub package_id: Uuid,
    pub current_state: RecommendationState,
    pub initial_confidence: Decimal,
    pub current_confidence: Decimal,
    pub success_rate: Decimal, // tracked once in production
    pub regret_analysis_score: Decimal, // tracked post-archive
}

impl RecommendationLifecycleTracker {
    pub fn new(package_id: Uuid, initial_confidence: Decimal) -> Self {
        Self {
            package_id,
            current_state: RecommendationState::Generated,
            initial_confidence,
            current_confidence: initial_confidence,
            success_rate: Decimal::new(0, 0),
            regret_analysis_score: Decimal::new(0, 0),
        }
    }

    pub fn transition_to(&mut self, next_state: RecommendationState) {
        // Enforce valid state transitions? For now just assign
        self.current_state = next_state;
    }

    pub fn calculate_drift(&self) -> Decimal {
        self.current_confidence - self.initial_confidence
    }

    pub fn update_confidence(&mut self, new_confidence: Decimal) {
        self.current_confidence = new_confidence;
    }
}
