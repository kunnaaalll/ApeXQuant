use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceUpdatedEvent {
    pub account_id: String,
    pub net_profit: Decimal,
    pub win_rate: Decimal,
    pub timestamp: u64,
}
