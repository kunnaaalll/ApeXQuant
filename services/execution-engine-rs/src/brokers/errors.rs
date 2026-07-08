use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum BrokerError {
    #[error("Broker is not connected: {0}")]
    NotConnected(String),
    #[error("Broker is currently degraded: {0}")]
    Degraded(String),
    #[error("Connection failure: {0}")]
    ConnectionFailure(String),
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
    #[error("Order submission failed: {0}")]
    OrderSubmissionFailed(String),
    #[error("Order modification failed: {0}")]
    OrderModificationFailed(String),
    #[error("Order cancellation failed: {0}")]
    OrderCancellationFailed(String),
    #[error("Position close failed: {0}")]
    PositionCloseFailed(String),
    #[error("Data validation error: {0}")]
    ValidationError(String),
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("Account not found or inaccessible: {0}")]
    AccountError(String),
    #[error("Internal broker error: {0}")]
    InternalError(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    #[error("Failover error: {0}")]
    FailoverError(String),
}
