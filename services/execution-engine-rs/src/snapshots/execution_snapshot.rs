use serde::{Deserialize, Serialize};

use crate::order::{OrderId, OrderSide, OrderStatus, OrderType, TimeInForce};
use crate::position::Position;
use crate::state::ExecutionState;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderSnapshot {
    pub order_id: OrderId,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub state: ExecutionState,
    pub size: Decimal,
    pub price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub time_in_force: TimeInForce,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub orders: Vec<OrderSnapshot>,
    pub positions: Vec<Position>,
}
