use serde::{Deserialize, Serialize};

use super::models::RecommendationExplanation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockOutcome {
    Allow,
    AllowReduced,
    Delay,
    Block,
    Freeze,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeAdmissionPolicy {
    pub outcome: BlockOutcome,
    pub score: u8,
    pub confidence: u8,
    pub reasons: Vec<String>,
    pub contributing_factors: Vec<String>,
    pub explanation: RecommendationExplanation,
}

pub struct BlockEngine;

impl BlockEngine {
    pub fn evaluate(
        heat_score: u8,
        health_score: u8,
        is_frozen: bool,
        is_critical_drawdown: bool,
    ) -> TradeAdmissionPolicy {
        if is_frozen {
            return TradeAdmissionPolicy {
                outcome: BlockOutcome::Freeze,
                score: 100,
                confidence: 100,
                reasons: vec!["Portfolio is in a frozen state".to_string()],
                contributing_factors: vec!["FrozenState".to_string()],
                explanation: RecommendationExplanation::new(
                    "All new trades are frozen due to extreme portfolio stress.",
                    "Portfolio entered a frozen state.",
                    "FrozenState",
                ),
            };
        }

        if is_critical_drawdown || heat_score >= 90 {
            return TradeAdmissionPolicy {
                outcome: BlockOutcome::Block,
                score: 90,
                confidence: 95,
                reasons: vec!["Critical drawdown or extreme heat".to_string()],
                contributing_factors: vec!["Drawdown".to_string(), "Heat".to_string()],
                explanation: RecommendationExplanation::new(
                    "Blocking new trades to prevent further capital loss.",
                    "Critical drawdown or extreme heat detected.",
                    "Drawdown/Heat",
                ),
            };
        }

        if heat_score >= 75 || health_score < 40 {
            return TradeAdmissionPolicy {
                outcome: BlockOutcome::Delay,
                score: 70,
                confidence: 85,
                reasons: vec!["Elevated heat or poor health".to_string()],
                contributing_factors: vec!["Heat".to_string(), "Health".to_string()],
                explanation: RecommendationExplanation::new(
                    "Delaying new trades until portfolio metrics improve.",
                    "Heat is elevated or health is poor.",
                    "Heat/Health",
                ),
            };
        }

        if heat_score >= 60 || health_score < 60 {
            return TradeAdmissionPolicy {
                outcome: BlockOutcome::AllowReduced,
                score: 40,
                confidence: 80,
                reasons: vec!["Moderate heat or suboptimal health".to_string()],
                contributing_factors: vec!["Heat".to_string(), "Health".to_string()],
                explanation: RecommendationExplanation::new(
                    "Allowing new trades with reduced sizing.",
                    "Heat is moderate or health is suboptimal.",
                    "Heat/Health",
                ),
            };
        }

        TradeAdmissionPolicy {
            outcome: BlockOutcome::Allow,
            score: 0,
            confidence: 95,
            reasons: vec!["Healthy portfolio state".to_string()],
            contributing_factors: vec!["Health".to_string(), "Heat".to_string()],
            explanation: RecommendationExplanation::new(
                "Allowing new trades, portfolio is healthy.",
                "Stable metrics.",
                "Health/Heat",
            ),
        }
    }
}
