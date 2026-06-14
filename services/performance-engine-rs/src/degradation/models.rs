use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradationMetrics {
    pub edge_decay: Decimal,
    pub expectancy_decay: Decimal,
    pub quality_deterioration: Decimal,
    pub stability_deterioration: Decimal,
    pub performance_drift: Decimal,
    pub duration: u32, // periods or ticks since decay started
    pub severity: Decimal,
    pub velocity: Decimal,
}

impl Default for DegradationMetrics {
    fn default() -> Self {
        Self {
            edge_decay: Decimal::ZERO,
            expectancy_decay: Decimal::ZERO,
            quality_deterioration: Decimal::ZERO,
            stability_deterioration: Decimal::ZERO,
            performance_drift: Decimal::ZERO,
            duration: 0,
            severity: Decimal::ZERO,
            velocity: Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradationAssessment {
    pub metrics: DegradationMetrics,
    pub state: super::states::DegradationState,
}
