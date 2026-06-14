use super::strategy_registry::StrategyProfile;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub winner: StrategyProfile,
    pub runner_up: Option<StrategyProfile>,
    pub loser: Option<StrategyProfile>,
    pub explanation: String,
}

pub struct StrategyComparisonEngine;

impl StrategyComparisonEngine {
    pub fn compare(mut strategies: Vec<StrategyProfile>) -> Option<ComparisonResult> {
        if strategies.is_empty() {
            return None;
        }
        if strategies.len() == 1 {
            return Some(ComparisonResult {
                winner: strategies[0].clone(),
                runner_up: None,
                loser: None,
                explanation: "Only one strategy provided.".to_string(),
            });
        }

        strategies.sort_by(|a, b| {
            let score_a = Self::calculate_score(a);
            let score_b = Self::calculate_score(b);
            score_b.cmp(&score_a) // descending
        });

        let winner = strategies[0].clone();
        let runner_up = if strategies.len() > 1 { Some(strategies[1].clone()) } else { None };
        let loser = if strategies.len() > 2 { Some(strategies.last().unwrap().clone()) } else { None };

        let explanation = format!(
            "Winner {} chosen due to highest combined score (expectancy, stability, drawdown, confidence).",
            winner.name
        );

        Some(ComparisonResult {
            winner,
            runner_up,
            loser,
            explanation,
        })
    }

    fn calculate_score(profile: &StrategyProfile) -> Decimal {
        use rust_decimal_macros::dec;
        let base = profile.expectancy * profile.confidence * profile.stability;
        let risk_adj = dec!(1) + profile.max_drawdown;
        if risk_adj == dec!(0) {
            base
        } else {
            base / risk_adj
        }
    }
}
