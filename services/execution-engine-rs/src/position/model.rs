use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::position_state::PositionState;
use crate::order::OrderSide;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(Uuid);

impl PositionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PositionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub position_id: PositionId,
    pub symbol: String,
    pub direction: OrderSide,
    pub entry_price: Decimal,
    pub size: Decimal,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub state: PositionState,
}

impl Position {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        direction: OrderSide,
        entry_price: Decimal,
        size: Decimal,
        stop_loss: Option<Decimal>,
        take_profit: Option<Decimal>,
    ) -> Self {
        Self {
            position_id: PositionId::new(),
            symbol,
            direction,
            entry_price,
            size,
            stop_loss,
            take_profit,
            state: PositionState::Opening,
        }
    }
}
