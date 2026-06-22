use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::PendingOrder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mt5Order {
    pub ticket: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub volume: Decimal,
    pub price: Decimal,
    pub status: String,
    pub timestamp: i64,
}

impl From<Mt5Order> for PendingOrder {
    fn from(val: Mt5Order) -> Self {
        PendingOrder {
            ticket: val.ticket,
            symbol: val.symbol,
            side: val.side,
            order_type: val.order_type,
            volume: val.volume,
            price: val.price,
            status: val.status,
            timestamp: val.timestamp,
        }
    }
}
