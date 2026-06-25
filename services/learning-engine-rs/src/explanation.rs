use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    pub subject: String,
    pub summary: String,
    pub primary_reason: String,
    pub supporting_metrics: Vec<String>,
}

pub struct ExplanationGenerator;

impl ExplanationGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_allocation_change(
        target: &str,
        action: &str,
        magnitude: &str,
        reason: &str,
    ) -> Explanation {
        Explanation {
            subject: "Allocation Change".to_string(),
            summary: format!("{} {} allocation by {}", action, target, magnitude),
            primary_reason: reason.to_string(),
            supporting_metrics: vec![],
        }
    }

    pub fn generate_retirement(
        strategy_id: &str,
        reason: &str,
        metrics: Vec<String>,
    ) -> Explanation {
        Explanation {
            subject: "Strategy Retirement".to_string(),
            summary: format!("Retire {}", strategy_id),
            primary_reason: reason.to_string(),
            supporting_metrics: metrics,
        }
    }

    pub fn generate_promotion(
        strategy_id: &str,
        level: &str,
        metrics: Vec<String>,
    ) -> Explanation {
        Explanation {
            subject: "Strategy Promotion".to_string(),
            summary: format!("Promote {} to {}", strategy_id, level),
            primary_reason: "Strategy met all promotion thresholds".to_string(),
            supporting_metrics: metrics,
        }
    }
}
