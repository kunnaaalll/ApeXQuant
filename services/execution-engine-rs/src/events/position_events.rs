use serde::{Deserialize, Serialize};

use crate::position::PositionId;
use crate::order::OrderSide;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionEvent {
    PositionOpened {
        position_id: PositionId,
        symbol: String,
        direction: OrderSide,
        entry_price: Decimal,
        size: Decimal,
        stop_loss: Option<Decimal>,
        take_profit: Option<Decimal>,
        timestamp: i64,
    },
    PositionClosed {
        position_id: PositionId,
        exit_price: Decimal,
        timestamp: i64,
    },
}
