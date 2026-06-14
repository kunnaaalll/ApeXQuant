use super::models::{ExecutionRequest, ExecutionResult, ExecutionResponse};
use crate::state_machine::states::OrderState;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct StopOrderHandler;

impl StopOrderHandler {
    pub fn process(req: &ExecutionRequest) -> Result<ExecutionResult, StopOrderError> {
        if req.quantity.is_sign_negative() || req.quantity.is_zero() {
            return Err(StopOrderError::InvalidQuantity);
        }
        if req.stop_price.is_none() {
            return Err(StopOrderError::MissingStopPrice);
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
pub enum StopOrderError {
    #[error("Invalid quantity for stop order")]
    InvalidQuantity,
    #[error("Missing stop price for stop order")]
    MissingStopPrice,
}
