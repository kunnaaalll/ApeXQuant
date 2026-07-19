use super::models::{
    CircuitBreakerState, DrawdownState, HiddenLeverage, RiskInputs, TailRiskScore,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreezeReason {
    pub source: String,
    pub severity: u32,
    pub timestamp: u64,
}

pub fn evaluate_freeze(inputs: &RiskInputs, current_time: u64) -> Option<FreezeReason> {
    if inputs.drawdown_state == DrawdownState::Frozen {
        return Some(FreezeReason {
            source: "Drawdown".to_string(),
            severity: 100,
            timestamp: current_time,
        });
    }

    if inputs.circuit_breaker_state == CircuitBreakerState::Frozen {
        return Some(FreezeReason {
            source: "CircuitBreaker".to_string(),
            severity: 100,
            timestamp: current_time,
        });
    }

    if inputs.tail_risk_score == TailRiskScore::Collapse {
        return Some(FreezeReason {
            source: "TailRisk".to_string(),
            severity: 90,
            timestamp: current_time,
        });
    }

    if inputs.hidden_leverage == HiddenLeverage::Collapse {
        return Some(FreezeReason {
            source: "HiddenLeverage".to_string(),
            severity: 90,
            timestamp: current_time,
        });
    }

    None
}
