use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use super::OrderType;
use crate::state_machine::states::OrderState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub order_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub order_id: String,
    pub broker_order_id: Option<String>,
    pub state: OrderState,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success(ExecutionResponse),
    Rejected { reason: String },
    Failed { error: String },
}
