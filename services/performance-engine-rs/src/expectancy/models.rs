use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectancyMetrics {
    pub wins: u32,
    pub losses: u32,
    pub breakevens: u32,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub average_rr: Decimal,
    pub trade_count: u32,
}

impl Default for ExpectancyMetrics {
    fn default() -> Self {
        Self {
            wins: 0,
            losses: 0,
            breakevens: 0,
            average_win: Decimal::ZERO,
            average_loss: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            average_rr: Decimal::ZERO,
            trade_count: 0,
        }
    }
}
