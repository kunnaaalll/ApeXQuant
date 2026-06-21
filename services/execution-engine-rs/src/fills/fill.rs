use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::order::OrderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FillId(Uuid);

impl FillId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for FillId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fill {
    pub fill_id: FillId,
    pub order_id: OrderId,
    pub price: Decimal,
    pub quantity: Decimal,
    pub commission: Decimal,
    pub slippage: Decimal,
}

impl Fill {
    pub fn new(
        order_id: OrderId,
        price: Decimal,
        quantity: Decimal,
        commission: Decimal,
        slippage: Decimal,
    ) -> Self {
        Self {
            fill_id: FillId::new(),
            order_id,
            price,
            quantity,
            commission,
            slippage,
        }
    }
}
