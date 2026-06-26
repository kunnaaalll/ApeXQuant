// src/analytics/efficiency.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use super::math::safe_clamp;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EfficiencyAssessment {
    pub capital_efficiency: Decimal,
    pub allocation_efficiency: Decimal,
    pub holding_efficiency: Decimal,
    pub risk_efficiency: Decimal,
    pub recovery_efficiency: Decimal,
}

impl EfficiencyAssessment {
    pub fn new(
        capital_efficiency: Decimal,
        allocation_efficiency: Decimal,
        holding_efficiency: Decimal,
        risk_efficiency: Decimal,
        recovery_efficiency: Decimal,
    ) -> Self {
        Self {
            capital_efficiency: safe_clamp(capital_efficiency, Decimal::ZERO, Decimal::MAX, Decimal::ZERO),
            allocation_efficiency: safe_clamp(allocation_efficiency, Decimal::ZERO, Decimal::MAX, Decimal::ZERO),
            holding_efficiency: safe_clamp(holding_efficiency, Decimal::ZERO, Decimal::MAX, Decimal::ZERO),
            risk_efficiency: safe_clamp(risk_efficiency, Decimal::ZERO, Decimal::MAX, Decimal::ZERO),
            recovery_efficiency: safe_clamp(recovery_efficiency, Decimal::ZERO, Decimal::MAX, Decimal::ZERO),
        }
    }
}
