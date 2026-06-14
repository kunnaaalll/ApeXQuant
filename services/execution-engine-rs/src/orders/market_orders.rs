use super::models::{ExecutionRequest, ExecutionResult, ExecutionResponse};
use crate::state_machine::states::OrderState;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MarketOrderHandler;

impl MarketOrderHandler {
    pub fn process(req: &ExecutionRequest) -> Result<ExecutionResult, MarketOrderError> {
        if req.quantity.is_sign_negative() || req.quantity.is_zero() {
            return Err(MarketOrderError::InvalidQuantity);
        }
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        
        Ok(ExecutionResult::Success(ExecutionResponse {
            order_id: req.order_id.clone(),
            broker_order_id: None,
            state: OrderState::Pending,
            timestamp: now,
        }))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarketOrderError {
    #[error("Invalid quantity for market order")]
    InvalidQuantity,
}
