use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mt5Fill {
    pub deal_ticket: String,
    pub order_ticket: String,
    pub symbol: String,
    pub side: String,
    pub volume: Decimal,
    pub price: Decimal,
    pub commission: Decimal,
    pub timestamp: i64,
}
