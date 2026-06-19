use super::models::{DrawdownState, ExposureState, HiddenLeverage, ReduceDecision, RiskInputs, TailRiskScore};

pub fn evaluate_reduction(inputs: &RiskInputs) -> ReduceDecision {
    if inputs.drawdown_state == DrawdownState::Frozen {
        return ReduceDecision::EmergencyReduction; // "Maximum reduction"
    }

    if inputs.drawdown_state == DrawdownState::Collapse
        || inputs.tail_risk_score == TailRiskScore::Collapse
        || inputs.hidden_leverage == HiddenLeverage::Collapse
        || inputs.exposure_state == ExposureState::Collapse
        || inputs.exposure_concentration == ExposureState::Collapse
    {
        return ReduceDecision::EmergencyReduction;
    }

    if inputs.drawdown_state == DrawdownState::Warning
        || inputs.tail_risk_score == TailRiskScore::Warning
        || inputs.hidden_leverage == HiddenLeverage::Warning
        || inputs.exposure_state == ExposureState::Warning
        || inputs.exposure_concentration == ExposureState::Warning
    {
        return ReduceDecision::ReduceSlightly;
    }

    ReduceDecision::NoAction
}
