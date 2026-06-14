use crate::orders::models::ExecutionRequest;

/// Dispatches orders to the external broker/bridge.
pub struct Dispatcher;

impl Dispatcher {
    pub async fn dispatch(&self, req: &ExecutionRequest) -> Result<(), DispatchError> {
        // Send order to broker
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("Failed to dispatch order to broker")]
    NetworkError,
}
