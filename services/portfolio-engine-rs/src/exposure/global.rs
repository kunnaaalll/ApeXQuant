use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalExposure {
    pub total_exposure: Decimal,
    pub net_exposure: Decimal,
    pub long_exposure: Decimal,
    pub short_exposure: Decimal,
    pub gross_exposure: Decimal,
    pub margin_utilization: Decimal,
    pub open_risk: Decimal,
    pub leverage: Decimal,
    pub position_count: usize,
}

impl Default for GlobalExposure {
    fn default() -> Self {
        Self {
            total_exposure: Decimal::ZERO,
            net_exposure: Decimal::ZERO,
            long_exposure: Decimal::ZERO,
            short_exposure: Decimal::ZERO,
            gross_exposure: Decimal::ZERO,
            margin_utilization: Decimal::ZERO,
            open_risk: Decimal::ZERO,
            leverage: Decimal::ZERO,
            position_count: 0,
        }
    }
}

impl GlobalExposure {
    pub fn new() -> Self {
        Self::default()
    }
}
