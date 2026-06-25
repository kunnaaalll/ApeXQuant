use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvaluation {
    pub decision_id: Uuid,
    pub is_successful: bool,
    pub action_quality: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementOutput {
    pub reward_score: Decimal,
    pub penalty_score: Decimal,
}

pub struct ReinforcementEngine;

impl Default for ReinforcementEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReinforcementEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, evaluation: &DecisionEvaluation) -> ReinforcementOutput {
        if evaluation.is_successful {
            ReinforcementOutput {
                reward_score: evaluation.action_quality,
                penalty_score: Decimal::ZERO,
            }
        } else {
            ReinforcementOutput {
                reward_score: Decimal::ZERO,
                // Inverse quality as penalty
                penalty_score: Decimal::ONE - evaluation.action_quality,
            }
        }
    }
}
