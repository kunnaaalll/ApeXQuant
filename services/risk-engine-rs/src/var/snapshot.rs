use super::events::VarRiskEvent;
use super::Severity;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarRiskSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub event: VarRiskEvent,
    pub historical_var_state: Decimal, // generic representation, could be more detailed
    pub parametric_var_state: Decimal,
    pub expected_shortfall_state: Decimal,
    pub tail_risk_state: Severity,
}

impl VarRiskSnapshot {
    pub fn new(
        version: u64,
        timestamp: i64,
        event: VarRiskEvent,
        historical_var_state: Decimal,
        parametric_var_state: Decimal,
        expected_shortfall_state: Decimal,
        tail_risk_state: Severity,
    ) -> Self {
        Self {
            version,
            timestamp,
            event,
            historical_var_state,
            parametric_var_state,
            expected_shortfall_state,
            tail_risk_state,
        }
    }
}
