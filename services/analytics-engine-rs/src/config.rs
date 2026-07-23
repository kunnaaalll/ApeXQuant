//! Analytics Engine Configuration

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    pub database_url: String,
    pub redis_url: String,
    pub eventbus_url: String,
    pub grpc_bind_addr: String,
    pub health_bind_addr: String,
    pub db_max_connections: u32,
    /// Batch size for PnL aggregation writes
    pub batch_size: usize,
}

impl AnalyticsConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: required_var("DATABASE_URL")?,
            redis_url: required_var("REDIS_URL")?,
            eventbus_url: optional_var("EVENTBUS_URL")
                .or_else(|| optional_var("EVENT_BUS_URL"))
                .ok_or_else(|| ConfigError::MissingEnvVar("EVENTBUS_URL".to_string()))?,
            grpc_bind_addr: optional_var("GRPC_BIND_ADDR")
                .unwrap_or_else(|| "0.0.0.0:50055".to_string()),
            health_bind_addr: optional_var("HEALTH_BIND_ADDR")
                .unwrap_or_else(|| "0.0.0.0:8084".to_string()),
            db_max_connections: optional_var("DB_MAX_CONNECTIONS")
                .map(|v| {
                    v.parse::<u32>().map_err(|e| {
                        ConfigError::InvalidValue("DB_MAX_CONNECTIONS".to_string(), e.to_string())
                    })
                })
                .transpose()?
                .unwrap_or(5),
            batch_size: optional_var("ANALYTICS_BATCH_SIZE")
                .map(|v| {
                    v.parse::<usize>().map_err(|e| {
                        ConfigError::InvalidValue("ANALYTICS_BATCH_SIZE".to_string(), e.to_string())
                    })
                })
                .transpose()?
                .unwrap_or(100),
        })
    }
}

fn required_var(name: &str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|_| ConfigError::MissingEnvVar(name.to_string()))
}

fn optional_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
