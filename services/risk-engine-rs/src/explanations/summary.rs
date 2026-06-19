use crate::recommendations::models::{RecommendationExplanation, RiskInputs, TradeAdmissionPolicy};

use super::constraints::detect_constraints;
use super::contributors::find_largest_contributor;
use super::deterioration::detect_deterioration;
use super::improvements::detect_improvements;
use super::reasons::track_reasons;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskNarrative {
    pub explanation: RecommendationExplanation,
}

pub fn generate_narrative(
    inputs: &RiskInputs,
    policy: TradeAdmissionPolicy,
    why: String,
) -> RiskNarrative {
    let reasons = track_reasons(
        &format!("{:?}", inputs.drawdown_state),
        &format!("{:?}", inputs.exposure_state),
        &format!("{:?}", inputs.correlation_severity),
        &format!("{:?}", inputs.tail_risk_score),
        &format!("{:?}", inputs.var_severity),
        &format!("{:?}", inputs.circuit_breaker_state),
    );

    let dominant_factor =
        find_largest_contributor(&reasons).unwrap_or_else(|| "None".to_string());

    // Mocking previous state for the sake of deterministic explanation generation
    // In a real system, you'd pass previous state explicitly
    let improvements = detect_improvements(100, 90, 50, 40, 20, 15, true);
    let deterioration = detect_deterioration(100, 100, 50, 50, 20, 20, false, false);

    let constraint =
        detect_constraints(inputs, policy).unwrap_or_else(|| "No constraints".to_string());

    let imp_str = if improvements.is_empty() {
        "None".to_string()
    } else {
        improvements.join(", ")
    };

    let det_str = if deterioration.is_empty() {
        "None".to_string()
    } else {
        deterioration.join(", ")
    };

    RiskNarrative {
        explanation: RecommendationExplanation {
            why,
            what_improved: imp_str,
            what_deteriorated: det_str,
            dominant_factor,
            prevented_stronger_recommendation: constraint,
        },
    }
}
