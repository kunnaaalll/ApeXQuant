use super::block::evaluate_blocking;
use super::freeze::evaluate_freeze;
use super::increase::evaluate_increase;
use super::models::{
    IncreaseDecision, RecommendationExplanation, ReduceDecision, RiskCommitteeDecision, RiskInputs,
    RiskRecommendation, TradeAdmissionPolicy,
};
use super::reduce::evaluate_reduction;

fn generate_explanation(
    inputs: &RiskInputs,
    recommendation: RiskRecommendation,
) -> RecommendationExplanation {
    let mut factors = Vec::new();
    let mut deterioration = Vec::new();

    if inputs.drawdown_state == super::models::DrawdownState::Collapse
        || inputs.drawdown_state == super::models::DrawdownState::Frozen
    {
        factors.push("Drawdown at collapse/frozen levels");
        deterioration.push("Severe drawdown");
    } else if inputs.drawdown_state == super::models::DrawdownState::Warning {
        deterioration.push("Drawdown warning");
    }

    if inputs.exposure_state == super::models::ExposureState::Collapse {
        factors.push("Exposure at collapse limits");
        deterioration.push("Exposure collapse");
    } else if inputs.exposure_state == super::models::ExposureState::Warning {
        deterioration.push("Exposure warning");
    }

    if inputs.correlation_severity == super::models::CorrelationSeverity::Critical {
        factors.push("Critical correlation severity");
        deterioration.push("High correlated risk");
    }

    if inputs.var_severity == super::models::VarSeverity::Critical {
        factors.push("Critical Value-at-Risk limits exceeded");
        deterioration.push("VaR threshold breach");
    }

    let dominant = if factors.is_empty() {
        "Standard operating conditions".to_string()
    } else {
        factors.join(", ")
    };

    let det_str = if deterioration.is_empty() {
        "None".to_string()
    } else {
        deterioration.join(", ")
    };

    let why = match recommendation {
        RiskRecommendation::FreezeTrading => format!("Trading frozen due to: {}", dominant),
        RiskRecommendation::EmergencyReduction => {
            format!("Emergency risk reduction mandated by: {}", dominant)
        }
        RiskRecommendation::ReduceAggressively => {
            format!("Aggressive risk reduction triggered by: {}", dominant)
        }
        RiskRecommendation::ReduceRisk => {
            format!("General risk reduction advised. Concerns: {}", det_str)
        }
        RiskRecommendation::MaintainRisk => {
            "Maintaining current risk parameters. No critical warnings.".to_string()
        }
        RiskRecommendation::IncreaseRisk => {
            "Conditions permit risk increase. All metrics healthy.".to_string()
        }
    };

    let improved = if recommendation == RiskRecommendation::IncreaseRisk {
        "Favorable market conditions and healthy portfolio metrics".to_string()
    } else {
        "None".to_string()
    };

    RecommendationExplanation {
        why,
        what_improved: improved,
        what_deteriorated: det_str,
        dominant_factor: dominant,
        prevented_stronger_recommendation: "Evaluation constraints".to_string(),
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
        (
            RiskRecommendation::FreezeTrading,
            TradeAdmissionPolicy::Freeze,
        )
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
        explanation: generate_explanation(inputs, recommendation),
        confidence: 100, // Fully deterministic rule-based
        timestamp: current_time,
    }
}
