
use serde::{Deserialize, Serialize};

use super::models::RecommendationExplanation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncreaseOutcome {
    Increase,
    Maintain,
    Delay,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncreaseExposureRecommendation {
    pub outcome: IncreaseOutcome,
    pub score: u8,
    pub confidence: u8,
    pub reasons: Vec<String>,
    pub contributing_factors: Vec<String>,
    pub explanation: RecommendationExplanation,
}

pub struct IncreaseExposureEngine;

impl IncreaseExposureEngine {
    pub fn evaluate(
        heat_score: u8,
        health_score: u8,
        quality_score: u8,
        is_frozen: bool,
        is_critical_drawdown: bool,
    ) -> IncreaseExposureRecommendation {
        if is_frozen {
            return IncreaseExposureRecommendation {
                outcome: IncreaseOutcome::Reject,
                score: 0,
                confidence: 100,
                reasons: vec!["Portfolio is frozen".to_string()],
                contributing_factors: vec!["FrozenState".to_string()],
                explanation: RecommendationExplanation::new(
                    "Trading is frozen, rejecting all increases to protect capital.",
                    "Portfolio entered a frozen state.",
                    "FrozenState",
                )
                .with_prevented("Any increase in exposure is strictly prohibited during a freeze."),
            };
        }

        if is_critical_drawdown {
            return IncreaseExposureRecommendation {
                outcome: IncreaseOutcome::Reject,
                score: 0,
                confidence: 100,
                reasons: vec!["Portfolio is in critical drawdown".to_string()],
                contributing_factors: vec!["Drawdown".to_string()],
                explanation: RecommendationExplanation::new(
                    "Trading is blocked, rejecting all increases to protect capital.",
                    "Portfolio entered a critical drawdown state.",
                    "Drawdown",
                )
                .with_prevented("Any increase in exposure is strictly prohibited during a critical drawdown."),
            };
        }

        if heat_score >= 80 {
            return IncreaseExposureRecommendation {
                outcome: IncreaseOutcome::Reject,
                score: 10,
                confidence: 90,
                reasons: vec!["Critical portfolio heat".to_string()],
                contributing_factors: vec!["Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Portfolio heat is critical, rejecting increases to avoid catastrophic risk.",
                    "Heat score exceeded safe limits.",
                    "Heat",
                )
                .with_prevented("Cannot increase exposure while heat is critical."),
            };
        }

        if health_score < 40 || quality_score < 40 {
            return IncreaseExposureRecommendation {
                outcome: IncreaseOutcome::Delay,
                score: 30,
                confidence: 85,
                reasons: vec!["Poor portfolio health or quality".to_string()],
                contributing_factors: vec!["Health".to_string(), "Quality".to_string()],
                explanation: RecommendationExplanation::new(
                    "Portfolio health or quality is poor, delaying increases.",
                    "Health or quality dropped below safe thresholds.",
                    "Health/Quality",
                )
                .with_prevented("Waiting for portfolio to stabilize before allowing increases."),
            };
        }

        if heat_score >= 60 {
            return IncreaseExposureRecommendation {
                outcome: IncreaseOutcome::Maintain,
                score: 50,
                confidence: 80,
                reasons: vec!["Elevated portfolio heat".to_string()],
                contributing_factors: vec!["Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Portfolio heat is elevated, maintaining current exposure.",
                    "Heat score is elevated but not critical.",
                    "Heat",
                )
                .with_prevented("Heat is too high for aggressive increases."),
            };
        }

        let score = ((health_score as u16 + quality_score as u16) / 2) as u8;

        IncreaseExposureRecommendation {
            outcome: IncreaseOutcome::Increase,
            score,
            confidence: 90,
            reasons: vec!["Strong portfolio health and quality, low heat".to_string()],
            contributing_factors: vec!["Health".to_string(), "Quality".to_string(), "Heat".to_string()],
            explanation: RecommendationExplanation::new(
                "Conditions are favorable for increasing exposure.",
                "Health, quality, and heat are all within optimal ranges.",
                "Health/Quality/Heat",
            )
            .with_improvements("Strong health and quality metrics."),
        }
    }
}
