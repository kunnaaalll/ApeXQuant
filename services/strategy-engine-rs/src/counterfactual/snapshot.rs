use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualSnapshot {
    pub strategy_id: String,
    pub original_profit: Decimal,
    pub simulated_profit: Decimal,
    pub timestamp: u64,
}
