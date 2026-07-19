#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DegradationState {
    #[default]
    Healthy,
    EarlyWarning,
    Weakening,
    Danger,
    Critical,
    Collapse,
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DegradationEngine {
    pub edge_decay: Decimal,
    pub expectancy_decay: Decimal,
    pub confidence_decay: Decimal,
}

impl DegradationEngine {
    pub fn new() -> Self {
        Self {
            edge_decay: Decimal::from(0),
            expectancy_decay: Decimal::from(0),
            confidence_decay: Decimal::from(0),
        }
    }

    pub fn evaluate(&self, decay: Decimal) -> DegradationState {
        if decay <= Decimal::from(-60) {
            DegradationState::Collapse
        } else if decay <= Decimal::from(-40) {
            DegradationState::Danger
        } else if decay <= Decimal::from(-20) {
            DegradationState::Weakening
        } else if decay < Decimal::from(0) {
            DegradationState::EarlyWarning
        } else {
            DegradationState::Healthy
        }
    }
}

impl Default for DegradationEngine {
    fn default() -> Self {
        Self::new()
    }
}
