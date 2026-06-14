use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionEvent {
    OrderFilled {
        position_id: Uuid,
        fill_price: Decimal,
        fill_size: Decimal,
    },
    MarketTick {
        position_id: Uuid,
        current_price: Decimal,
    },
    ScaleInRequested {
        position_id: Uuid,
        additional_size: Decimal,
    },
    ScaleOutRequested {
        position_id: Uuid,
        reduce_size: Decimal,
    },
    CloseRequested {
        position_id: Uuid,
        reason: String,
    },
    PositionClosed {
        position_id: Uuid,
        exit_price: Decimal,
    },
}
