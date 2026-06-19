use super::models::{CircuitBreakerState, CorrelationSeverity, DrawdownState, RiskInputs, TradeAdmissionPolicy, VarSeverity};

pub fn evaluate_blocking(inputs: &RiskInputs) -> TradeAdmissionPolicy {
    if inputs.drawdown_state == DrawdownState::Frozen
        || inputs.circuit_breaker_state == CircuitBreakerState::Frozen
    {
        return TradeAdmissionPolicy::Freeze;
    }

    if inputs.var_severity == VarSeverity::Critical {
        return TradeAdmissionPolicy::Block;
    }

    if inputs.correlation_severity == CorrelationSeverity::High
        || inputs.correlation_severity == CorrelationSeverity::Critical
    {
        return TradeAdmissionPolicy::Delay;
    }

    TradeAdmissionPolicy::Allow
}
