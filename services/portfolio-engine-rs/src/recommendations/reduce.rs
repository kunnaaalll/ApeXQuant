use serde::{Deserialize, Serialize};

use super::models::RecommendationExplanation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReduceOutcome {
    NoAction,
    ReduceSlightly,
    ReduceModerately,
    ReduceAggressively,
    EmergencyReduction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionAssessment {
    pub outcome: ReduceOutcome,
    pub score: u8,
    pub confidence: u8,
    pub reasons: Vec<String>,
    pub contributing_factors: Vec<String>,
    pub explanation: RecommendationExplanation,
}

pub struct ReduceExposureEngine;

impl ReduceExposureEngine {
    pub fn evaluate(
        heat_score: u8,
        health_score: u8,
        is_frozen: bool,
        is_critical_drawdown: bool,
    ) -> ReductionAssessment {
        if is_frozen || is_critical_drawdown {
            return ReductionAssessment {
                outcome: ReduceOutcome::EmergencyReduction,
                score: 100,
                confidence: 100,
                reasons: vec!["Portfolio is frozen or in critical drawdown".to_string()],
                contributing_factors: vec!["FrozenState".to_string(), "Drawdown".to_string()],
                explanation: RecommendationExplanation::new(
                    "Emergency reduction required to protect remaining capital.",
                    "Portfolio entered a frozen or critical drawdown state.",
                    "FrozenState/Drawdown",
                ),
            };
        }

        if heat_score >= 90 {
            return ReductionAssessment {
                outcome: ReduceOutcome::ReduceAggressively,
                score: 85,
                confidence: 90,
                reasons: vec!["Extreme portfolio heat".to_string()],
                contributing_factors: vec!["Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Aggressive reduction required due to extreme heat.",
                    "Heat score exceeded critical limits.",
                    "Heat",
                ),
            };
        }

        if health_score < 30 || heat_score >= 75 {
            return ReductionAssessment {
                outcome: ReduceOutcome::ReduceModerately,
                score: 65,
                confidence: 85,
                reasons: vec!["Poor portfolio health or elevated heat".to_string()],
                contributing_factors: vec!["Health".to_string(), "Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Moderate reduction required due to poor health or elevated heat.",
                    "Health dropped or heat rose to concerning levels.",
                    "Health/Heat",
                ),
            };
        }

        if health_score < 50 || heat_score >= 60 {
            return ReductionAssessment {
                outcome: ReduceOutcome::ReduceSlightly,
                score: 40,
                confidence: 80,
                reasons: vec!["Suboptimal portfolio health or slightly elevated heat".to_string()],
                contributing_factors: vec!["Health".to_string(), "Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Slight reduction required due to suboptimal health or slightly elevated heat.",
                    "Health or heat metrics are showing mild deterioration.",
                    "Health/Heat",
                ),
            };
        }

        ReductionAssessment {
            outcome: ReduceOutcome::NoAction,
            score: 0,
            confidence: 95,
            reasons: vec!["Healthy portfolio state".to_string()],
            contributing_factors: vec!["Health".to_string()],
            explanation: RecommendationExplanation::new(
                "No reduction required, portfolio is healthy.",
                "Stable metrics.",
                "Health",
            ),
        }
    }
}
