// src/analytics/efficiency.rs
use serde::{Deserialize, Serialize};
use super::math::safe_clamp;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EfficiencyAssessment {
    pub capital_efficiency: f64,
    pub allocation_efficiency: f64,
    pub holding_efficiency: f64,
    pub risk_efficiency: f64,
    pub recovery_efficiency: f64,
}

impl EfficiencyAssessment {
    pub fn new(
        capital_efficiency: f64,
        allocation_efficiency: f64,
        holding_efficiency: f64,
        risk_efficiency: f64,
        recovery_efficiency: f64,
    ) -> Self {
        Self {
            capital_efficiency: safe_clamp(capital_efficiency, 0.0, f64::MAX, 0.0),
            allocation_efficiency: safe_clamp(allocation_efficiency, 0.0, f64::MAX, 0.0),
            holding_efficiency: safe_clamp(holding_efficiency, 0.0, f64::MAX, 0.0),
            risk_efficiency: safe_clamp(risk_efficiency, 0.0, f64::MAX, 0.0),
            recovery_efficiency: safe_clamp(recovery_efficiency, 0.0, f64::MAX, 0.0),
        }
    }
}
