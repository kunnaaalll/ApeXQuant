use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownSnapshot {
    pub current_drawdown: Decimal,
    pub max_drawdown: Decimal,
    pub peak_equity: Decimal,
    pub timestamp: u64,
}
