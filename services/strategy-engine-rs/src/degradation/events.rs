use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationEvent {
    pub strategy_id: String,
    pub expectancy_decline: Decimal,
    pub profit_factor_decline: Decimal,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseDetectedEvent {
    pub strategy_id: String,
    pub drawdown: Decimal,
    pub timestamp: u64,
}
