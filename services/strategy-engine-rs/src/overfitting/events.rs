use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfittingDetectedEvent {
    pub strategy_id: String,
    pub overfit_ratio: Decimal,
    pub timestamp: u64,
}
