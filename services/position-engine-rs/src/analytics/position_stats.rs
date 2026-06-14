use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionStatistics {
    pub average_duration_secs: u64,
    pub win_rate: f32,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
}

impl PositionStatistics {
    pub fn default() -> Self {
        Self {
            average_duration_secs: 0,
            win_rate: 0.0,
            profit_factor: Decimal::ZERO,
            expectancy: Decimal::ZERO,
        }
    }
}
