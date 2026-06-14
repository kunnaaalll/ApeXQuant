use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StabilityMetrics {
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub ulcer_index: Decimal,
    pub recovery_factor: Decimal,
    pub consistency: Decimal,
    pub variance: Decimal,
    pub stability_score: Decimal,
}

impl Default for StabilityMetrics {
    fn default() -> Self {
        Self {
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            calmar_ratio: Decimal::ZERO,
            ulcer_index: Decimal::ZERO,
            recovery_factor: Decimal::ZERO,
            consistency: Decimal::ZERO,
            variance: Decimal::ZERO,
            stability_score: Decimal::ZERO,
        }
    }
}
