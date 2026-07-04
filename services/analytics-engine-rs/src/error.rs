//! Analytics Engine Error Types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("Config error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Metrics computation error: {0}")]
    Metrics(String),
}
