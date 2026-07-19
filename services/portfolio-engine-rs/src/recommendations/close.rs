use serde::{Deserialize, Serialize};

use super::models::RecommendationExplanation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseOutcome {
    Hold,
    CloseWeakPositions,
    CloseCorrelatedPositions,
    CloseHighRiskPositions,
    EmergencyLiquidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseAssessment {
    pub outcome: CloseOutcome,
    pub score: u8,
    pub confidence: u8,
    pub reasons: Vec<String>,
    pub contributing_factors: Vec<String>,
    pub explanation: RecommendationExplanation,
}

pub struct CloseExposureEngine;

impl CloseExposureEngine {
    pub fn evaluate(
        heat_score: u8,
        health_score: u8,
        is_frozen: bool,
        is_critical_drawdown: bool,
    ) -> CloseAssessment {
        if is_frozen || is_critical_drawdown || heat_score >= 95 {
            return CloseAssessment {
                outcome: CloseOutcome::EmergencyLiquidation,
                score: 100,
                confidence: 100,
                reasons: vec![
                    "Critical portfolio state requiring immediate liquidation".to_string()
                ],
                contributing_factors: vec![
                    "FrozenState".to_string(),
                    "Drawdown".to_string(),
                    "Heat".to_string(),
                ],
                explanation: RecommendationExplanation::new(
                    "Emergency liquidation required to prevent total capital loss.",
                    "Portfolio entered a frozen, critical drawdown, or extreme heat state.",
                    "FrozenState/Drawdown/Heat",
                ),
            };
        }

        if heat_score >= 85 {
            return CloseAssessment {
                outcome: CloseOutcome::CloseHighRiskPositions,
                score: 80,
                confidence: 90,
                reasons: vec!["High portfolio heat".to_string()],
                contributing_factors: vec!["Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Close high-risk positions to reduce portfolio heat.",
                    "Heat score exceeded safe limits.",
                    "Heat",
                ),
            };
        }

        if health_score < 40 {
            return CloseAssessment {
                outcome: CloseOutcome::CloseWeakPositions,
                score: 60,
                confidence: 85,
                reasons: vec!["Poor portfolio health".to_string()],
                contributing_factors: vec!["Health".to_string()],
                explanation: RecommendationExplanation::new(
                    "Close weak positions to improve overall portfolio health.",
                    "Health score dropped below safe thresholds.",
                    "Health",
                ),
            };
        }

        CloseAssessment {
            outcome: CloseOutcome::Hold,
            score: 0,
            confidence: 95,
            reasons: vec!["Stable portfolio state".to_string()],
            contributing_factors: vec!["Health".to_string(), "Heat".to_string()],
            explanation: RecommendationExplanation::new(
                "No closures required, portfolio is stable.",
                "Stable metrics.",
                "Health/Heat",
            ),
        }
    }
}
