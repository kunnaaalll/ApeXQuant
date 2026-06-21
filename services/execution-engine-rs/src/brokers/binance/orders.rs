use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::brokers::responses::PendingOrder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinanceOrder {
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: String,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub side: String,
    pub r#type: String,
    pub update_time: i64,
}

impl Into<PendingOrder> for BinanceOrder {
    fn into(self) -> PendingOrder {
        PendingOrder {
            ticket: self.order_id.to_string(),
            symbol: self.symbol,
            side: self.side,
            order_type: self.r#type,
            volume: self.orig_qty - self.executed_qty,
            price: self.price,
            status: self.status,
            timestamp: self.update_time,
        }
    }
}
