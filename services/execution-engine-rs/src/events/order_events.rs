use serde::{Deserialize, Serialize};

use crate::fills::FillId;
use crate::order::{OrderId, OrderSide, OrderType, TimeInForce};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderEvent {
    OrderCreated {
        order_id: OrderId,
        symbol: String,
        order_type: OrderType,
        side: OrderSide,
        size: Decimal,
        price: Option<Decimal>,
        time_in_force: TimeInForce,
        timestamp: i64,
    },
    OrderSubmitted {
        order_id: OrderId,
        timestamp: i64,
    },
    OrderFilled {
        order_id: OrderId,
        fill_id: FillId,
        filled_price: Decimal,
        filled_quantity: Decimal,
        timestamp: i64,
    },
    OrderCancelled {
        order_id: OrderId,
        reason: Option<String>,
        timestamp: i64,
    },
}
