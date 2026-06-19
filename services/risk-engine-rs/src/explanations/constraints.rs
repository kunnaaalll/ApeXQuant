use crate::recommendations::models::{
    CorrelationSeverity, DrawdownState, RiskInputs, TradeAdmissionPolicy, VarSeverity,
};

pub fn detect_constraints(
    inputs: &RiskInputs,
    current_policy: TradeAdmissionPolicy,
) -> Option<String> {
    if inputs.var_severity == VarSeverity::Critical {
        return Some("Critical VaR prevented increase.".to_string());
    }

    if inputs.correlation_severity == CorrelationSeverity::High
        || inputs.correlation_severity == CorrelationSeverity::Critical
    {
        return Some("High correlation prevented aggressive sizing.".to_string());
    }

    if inputs.drawdown_state == DrawdownState::Warning
        || inputs.drawdown_state == DrawdownState::Collapse
        || inputs.drawdown_state == DrawdownState::Frozen
    {
        return Some("Drawdown state restricted capital.".to_string());
    }

    if current_policy == TradeAdmissionPolicy::Delay {
        return Some("Admission policy delayed prevented immediate scaling.".to_string());
    }

    None
}
