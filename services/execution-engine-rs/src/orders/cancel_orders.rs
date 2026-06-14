use super::models::{ExecutionRequest, ExecutionResult, ExecutionResponse};
use crate::state_machine::states::OrderState;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CancelOrderHandler;

impl CancelOrderHandler {
    pub fn process(req: &ExecutionRequest) -> Result<ExecutionResult, CancelOrderError> {
        // Validation logic for cancellations
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        
        Ok(ExecutionResult::Success(ExecutionResponse {
            order_id: req.order_id.clone(),
            broker_order_id: None,
            state: OrderState::Cancelled, // Immediate cancellation request
            timestamp: now,
        }))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CancelOrderError {
    #[error("Failed to cancel order")]
    CancellationFailed,
}
