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

impl Into<PendingOrder> for Mt5Order {
    fn into(self) -> PendingOrder {
        PendingOrder {
            ticket: self.ticket,
            symbol: self.symbol,
            side: self.side,
            order_type: self.order_type,
            volume: self.volume,
            price: self.price,
            status: self.status,
            timestamp: self.timestamp,
        }
    }
}
