use super::models::{ExecutionRequest, ExecutionResult, ExecutionResponse};
use crate::state_machine::states::OrderState;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LimitOrderHandler;

impl LimitOrderHandler {
    pub fn process(req: &ExecutionRequest) -> Result<ExecutionResult, LimitOrderError> {
        if req.quantity.is_sign_negative() || req.quantity.is_zero() {
            return Err(LimitOrderError::InvalidQuantity);
        }
        if req.price.is_none() {
            return Err(LimitOrderError::MissingPrice);
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
pub enum LimitOrderError {
    #[error("Invalid quantity for limit order")]
    InvalidQuantity,
    #[error("Missing price for limit order")]
    MissingPrice,
}
