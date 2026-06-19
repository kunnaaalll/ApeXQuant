use super::block::evaluate_blocking;
use super::freeze::evaluate_freeze;
use super::increase::evaluate_increase;
use super::models::{
    IncreaseDecision, RecommendationExplanation, ReduceDecision, RiskCommitteeDecision, RiskInputs,
    RiskRecommendation, TradeAdmissionPolicy,
};
use super::reduce::evaluate_reduction;

// A simple mock for generating explanations.
// In a full implementation, this could call out to the explanation module.
fn generate_mock_explanation(recommendation: RiskRecommendation) -> RecommendationExplanation {
    RecommendationExplanation {
        why: format!("Generated for {:?}", recommendation),
        what_improved: "None".to_string(),
        what_deteriorated: "None".to_string(),
        dominant_factor: "Calculated".to_string(),
        prevented_stronger_recommendation: "System constraints".to_string(),
    }
}

pub fn evaluate_committee(inputs: &RiskInputs, current_time: u64) -> RiskCommitteeDecision {
    let freeze = evaluate_freeze(inputs, current_time);
    let block = evaluate_blocking(inputs);
    let reduce = evaluate_reduction(inputs);
    let increase = evaluate_increase(inputs);

    // Priority hierarchy:
    // Freeze > Emergency Reduction > Aggressive Reduction > Maintain > Increase

    let (recommendation, admission_policy) = if freeze.is_some() {
        (RiskRecommendation::FreezeTrading, TradeAdmissionPolicy::Freeze)
    } else if reduce == ReduceDecision::EmergencyReduction {
        // Emergency reduction always maps to severe block or delay
        let policy = if block == TradeAdmissionPolicy::Freeze {
            TradeAdmissionPolicy::Freeze
        } else {
            TradeAdmissionPolicy::Block
        };
        (RiskRecommendation::EmergencyReduction, policy)
    } else if reduce == ReduceDecision::ReduceAggressively {
        (RiskRecommendation::ReduceAggressively, block)
    } else if reduce == ReduceDecision::ReduceModerately || reduce == ReduceDecision::ReduceSlightly
    {
        (RiskRecommendation::ReduceRisk, block)
    } else if increase == IncreaseDecision::Increase {
        (RiskRecommendation::IncreaseRisk, block)
    } else if increase == IncreaseDecision::Maintain {
        (RiskRecommendation::MaintainRisk, block)
    } else {
        // Fallback if rejected/delayed increase but no reduction needed
        (RiskRecommendation::MaintainRisk, block)
    };

    RiskCommitteeDecision {
        recommendation,
        admission_policy,
        explanation: generate_mock_explanation(recommendation),
        confidence: 100, // Fully deterministic rule-based
        timestamp: current_time,
    }
}
