use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::order::{OrderId, OrderSide, OrderType, TimeInForce};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub order_id: OrderId,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub size: Decimal,
    pub price: Option<Decimal>,
    pub time_in_force: TimeInForce,
}
