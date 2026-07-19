use super::models::{
    CircuitBreakerState, CorrelationSeverity, DrawdownState, IncreaseDecision, RiskInputs,
    VarSeverity,
};

pub fn evaluate_increase(inputs: &RiskInputs) -> IncreaseDecision {
    if inputs.drawdown_state == DrawdownState::Frozen
        || inputs.circuit_breaker_state == CircuitBreakerState::Frozen
    {
        return IncreaseDecision::Reject;
    }

    if inputs.var_severity == VarSeverity::Critical {
        return IncreaseDecision::Reject;
    }

    if inputs.correlation_severity == CorrelationSeverity::Critical {
        return IncreaseDecision::Delay;
    }

    if inputs.circuit_breaker_state == CircuitBreakerState::Restricted {
        return IncreaseDecision::Maintain;
    }

    IncreaseDecision::Increase
}
