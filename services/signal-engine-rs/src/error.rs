//! Error types for the Signal Engine

use thiserror::Error;

/// Result type alias with SignalEngineError
pub type Result<T> = std::result::Result<T, SignalEngineError>;

/// Signal Engine errors
#[derive(Error, Debug)]
pub enum SignalEngineError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Invalid market data
    #[error("Invalid market data: {0}")]
    InvalidData(String),

    /// Missing timeframe data
    #[error("Missing timeframe data: {timeframe}")]
    MissingTimeframe {
        /// The missing timeframe
        timeframe: String,
    },

    /// Data validation failed
    #[error("Data validation failed: {0}")]
    Validation(String),

    /// Pattern detection error
    #[error("Pattern detection failed: {0}")]
    PatternDetection(String),

    /// Calculation error
    #[error("Calculation error: {0}")]
    Calculation(String),

    /// Storage/IO error
    #[error("Storage error: {0}")]
    Storage(String),

    /// gRPC communication error
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl SignalEngineError {
    /// Create a new invalid data error
    pub fn invalid_data<S: Into<String>>(msg: S) -> Self {
        Self::InvalidData(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new pattern detection error
    pub fn pattern_detection<S: Into<String>>(msg: S) -> Self {
        Self::PatternDetection(msg.into())
    }

    /// Create a new calculation error
    pub fn calculation<S: Into<String>>(msg: S) -> Self {
        Self::Calculation(msg.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}
