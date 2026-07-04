use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfittingSnapshot {
    pub strategy_id: String,
    pub overfit_ratio: Decimal,
    pub complexity_score: u32,
    pub timestamp: u64,
}
