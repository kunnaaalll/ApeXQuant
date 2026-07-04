use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownSnapshot {
    pub current_drawdown: Decimal,
    pub max_drawdown: Decimal,
    pub peak_equity: Decimal,
    pub timestamp: u64,
}
