use thiserror::Error;

/// Errors that can occur in the Risk Engine
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RiskError {
    /// Insufficient margin for proposed position
    #[error("Insufficient margin: required={required}, available={available}")]
    InsufficientMargin { required: String, available: String },

    /// Maximum position limit reached
    #[error("Maximum position limit reached: {0} positions open")]
    MaxPositionsReached(u32),

    /// Daily loss limit exceeded
    #[error("Daily loss limit exceeded: loss={loss}, limit={limit}")]
    DailyLossLimitExceeded { loss: String, limit: String },

    /// Drawdown limit exceeded
    #[error("Drawdown limit exceeded: current={current}, limit={limit}")]
    DrawdownLimitExceeded { current: String, limit: String },

    /// Circuit breaker triggered
    #[error("Circuit breaker triggered: {breaker}")]
    CircuitBreakerTriggered { breaker: String },

    /// Invalid position size calculation
    #[error("Invalid position size: {reason}")]
    InvalidPositionSize { reason: String },

    /// Invalid input parameters
    #[error("Invalid input: {field} - {reason}")]
    InvalidInput { field: String, reason: String },

    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    /// Calculation overflow
    #[error("Calculation overflow in {operation}")]
    CalculationOverflow { operation: String },

    /// Exposure limit exceeded
    #[error("Exposure limit exceeded: currency={currency}, exposure={exposure}, limit={limit}")]
    ExposureLimitExceeded {
        currency: String,
        exposure: String,
        limit: String,
    },

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    Validation(String),
}

impl RiskError {
    /// Create a storage error
    pub fn storage<E: std::fmt::Display>(e: E) -> Self {
        Self::Storage(e.to_string())
    }

    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(field: S, reason: S) -> Self {
        Self::InvalidInput {
            field: field.into(),
            reason: reason.into(),
        }
    }
}
