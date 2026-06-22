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

impl From<BinanceOrder> for PendingOrder {
    fn from(val: BinanceOrder) -> Self {
        PendingOrder {
            ticket: val.order_id.to_string(),
            symbol: val.symbol,
            side: val.side,
            order_type: val.r#type,
            volume: val.orig_qty - val.executed_qty,
            price: val.price,
            status: val.status,
            timestamp: val.update_time,
        }
    }
}
