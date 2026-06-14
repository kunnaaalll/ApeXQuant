use crate::state_machine::OrderState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalState {
    pub tracked_orders: Vec<InternalOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalOrder {
    pub id: String,
    pub state: OrderState,
    pub volume: rust_decimal::Decimal,
}
