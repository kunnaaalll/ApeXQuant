pub mod block;
pub mod committee;
pub mod consistency;
pub mod events;
pub mod freeze;
pub mod increase;
pub mod models;
pub mod reduce;
pub mod snapshot;

#[cfg(test)]
pub mod tests;

pub use committee::evaluate_committee;
pub use models::{RiskCommitteeDecision, RiskInputs, RiskRecommendation, TradeAdmissionPolicy};

use rust_decimal::Decimal;

// ─── Recommendation Engine Facade ────────────────────────────────────────────

/// Thin stateful wrapper over the functional committee evaluation.
/// Stores the most-recent committee decision for gRPC layer reads.
#[derive(Debug, Clone)]
pub struct RiskRecommendationEngine {
    last_decision: Option<RiskCommitteeDecision>,
}

/// Simplified view exposed to the gRPC layer.
#[derive(Debug, Clone)]
pub struct CurrentRiskRecommendation {
    pub action: RiskRecommendation,
    pub confidence: Decimal,
    pub reason: String,
}

impl RiskRecommendationEngine {
    pub fn new() -> Self {
        Self {
            last_decision: None,
        }
    }

    /// Run the committee and cache the result.
    pub fn evaluate(&mut self, inputs: &RiskInputs) {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        self.last_decision = Some(evaluate_committee(inputs, now_ms));
    }

    /// Return the current recommendation for the gRPC layer.
    pub fn current(&self) -> CurrentRiskRecommendation {
        match &self.last_decision {
            Some(d) => CurrentRiskRecommendation {
                action: d.recommendation,
                confidence: Decimal::from(d.confidence),
                reason: d.explanation.why.clone(),
            },
            None => CurrentRiskRecommendation {
                action: RiskRecommendation::MaintainRisk,
                confidence: Decimal::ZERO,
                reason: "no_evaluation_yet".to_owned(),
            },
        }
    }
}

impl Default for RiskRecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}
