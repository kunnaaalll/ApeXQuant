use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AllocationError {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Capacity exceeded: {reason}")]
    CapacityExceeded { reason: String },

    #[error("Negative reserve amount: {0}")]
    NegativeReserve(String),

    #[error("Invalid allocation size: {0}")]
    InvalidAllocationSize(String),
}
