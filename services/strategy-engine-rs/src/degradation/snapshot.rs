use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationSnapshot {
    pub strategy_id: String,
    pub current_drawdown: Decimal,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub timestamp: u64,
}
