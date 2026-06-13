//! Error types for the Event Bus

use std::fmt;
use thiserror::Error;

/// Result type alias for Event Bus operations
pub type Result<T> = std::result::Result<T, EventBusError>;

/// Error types for Event Bus operations
#[derive(Error, Debug, Clone)]
pub enum EventBusError {
    /// Redis connection error
    #[error("Redis error: {0}")]
    Redis(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Event validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    Subscription(String),

    /// Publish error
    #[error("Publish error: {0}")]
    Publish(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    Unavailable(String),

    /// Backpressure - too many pending events
    #[error("Backpressure: maximum pending events exceeded")]
    Backpressure,

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl EventBusError {
    /// Returns true if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            EventBusError::Redis(_)
                | EventBusError::Timeout
                | EventBusError::Unavailable(_)
                | EventBusError::Backpressure
        )
    }

    /// Returns the error code string
    pub fn code(&self) -> &'static str {
        match self {
            EventBusError::Redis(_) => "REDIS_ERROR",
            EventBusError::Serialization(_) => "SERIALIZATION_ERROR",
            EventBusError::Deserialization(_) => "DESERIALIZATION_ERROR",
            EventBusError::Validation(_) => "VALIDATION_ERROR",
            EventBusError::Subscription(_) => "SUBSCRIPTION_ERROR",
            EventBusError::Publish(_) => "PUBLISH_ERROR",
            EventBusError::Storage(_) => "STORAGE_ERROR",
            EventBusError::Config(_) => "CONFIG_ERROR",
            EventBusError::Timeout => "TIMEOUT",
            EventBusError::Unavailable(_) => "UNAVAILABLE",
            EventBusError::Backpressure => "BACKPRESSURE",
            EventBusError::Unknown(_) => "UNKNOWN",
        }
    }

    /// Returns HTTP status code equivalent
    pub fn http_status(&self) -> u16 {
        match self {
            EventBusError::Redis(_) => 503,
            EventBusError::Timeout => 504,
            EventBusError::Unavailable(_) => 503,
            EventBusError::Backpressure => 429,
            EventBusError::Validation(_) => 400,
            EventBusError::Config(_) => 500,
            _ => 500,
        }
    }
}

impl From<redis::RedisError> for EventBusError {
    fn from(err: redis::RedisError) -> Self {
        EventBusError::Redis(err.to_string())
    }
}

impl From<serde_json::Error> for EventBusError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() {
            EventBusError::Deserialization(err.to_string())
        } else {
            EventBusError::Serialization(err.to_string())
        }
    }
}

impl From<std::io::Error> for EventBusError {
    fn from(err: std::io::Error) -> Self {
        EventBusError::Storage(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for EventBusError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        EventBusError::Timeout
    }
}

/// Error context for enriched error messages
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: &'static str,
    pub topic: Option<String>,
    pub event_id: Option<String>,
    pub details: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: &'static str) -> Self {
        Self {
            operation,
            topic: None,
            event_id: None,
            details: None,
        }
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn with_event_id(mut self, event_id: impl Into<String>) -> Self {
        self.event_id = Some(event_id.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation={}", self.operation)?;
        if let Some(topic) = &self.topic {
            write!(f, ", topic={}", topic)?;
        }
        if let Some(event_id) = &self.event_id {
            write!(f, ", event_id={}", event_id)?;
        }
        if let Some(details) = &self.details {
            write!(f, ", details={}", details)?;
        }
        Ok(())
    }
}
