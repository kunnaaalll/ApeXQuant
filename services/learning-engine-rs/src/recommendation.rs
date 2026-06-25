use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationAction {
    IncreaseAllocation(Decimal),
    ReduceAllocation(Decimal),
    FreezeStrategy,
    RetireStrategy,
    BeginShadowTesting,
    PromoteStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecommendation {
    pub target_id: Uuid,
    pub action: RecommendationAction,
    pub rationale: String,
}

pub struct RecommendationEngine;

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate_strategy(
        &self,
        strategy_id: Uuid,
        confidence_score: u8,
        win_rate: Decimal,
        decay_score: Decimal,
    ) -> Option<LearningRecommendation> {
        if decay_score > Decimal::new(75, 2) {
            return Some(LearningRecommendation {
                target_id: strategy_id,
                action: RecommendationAction::RetireStrategy,
                rationale: "High decay score indicating broken edge".to_string(),
            });
        }
        
        if confidence_score > 80 && win_rate > Decimal::new(55, 2) {
            return Some(LearningRecommendation {
                target_id: strategy_id,
                action: RecommendationAction::PromoteStrategy,
                rationale: "High confidence and strong win rate".to_string(),
            });
        }

        if win_rate < Decimal::new(40, 2) {
            return Some(LearningRecommendation {
                target_id: strategy_id,
                action: RecommendationAction::ReduceAllocation(Decimal::new(25, 2)),
                rationale: "Poor win rate, reducing allocation by 25%".to_string(),
            });
        }

        None
    }
}
