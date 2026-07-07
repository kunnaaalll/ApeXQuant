//! Risk Engine Configuration
//!
//! Loads all connection strings and tuning parameters from environment variables.
//! No hardcoded connection strings are permitted.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid value for environment variable {0}: {1}")]
    InvalidValue(String, String),
}

/// Risk engine configuration loaded entirely from environment.
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// PostgreSQL connection URL (env: DATABASE_URL)
    pub database_url: String,
    /// Redis connection URL (env: REDIS_URL)
    pub redis_url: String,
    /// Broker gateway URL (env: BROKER_URL)
    pub broker_url: String,
    /// Event bus gRPC URL (env: EVENT_BUS_URL)
    pub eventbus_url: String,
    /// Maximum PostgreSQL connection pool size (env: DB_MAX_CONNECTIONS, default: 10)
    pub db_max_connections: u32,
    /// gRPC server bind address (env: GRPC_BIND_ADDR, default: 0.0.0.0:50051)
    pub grpc_bind_addr: String,
    /// Health check HTTP bind address (env: HEALTH_BIND_ADDR, default: 0.0.0.0:8080)
    pub health_bind_addr: String,
}

impl RiskConfig {
    /// Load configuration from environment variables.
    ///
    /// Returns `ConfigError::MissingEnvVar` for any required variable that is absent.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = required_var("DATABASE_URL")?;
        let redis_url = required_var("REDIS_URL")?;
        let broker_url = required_var("BROKER_URL")?;
        let eventbus_url = required_var("EVENT_BUS_URL")?;

        let db_max_connections = optional_var("DB_MAX_CONNECTIONS")
            .map(|v| v.parse::<u32>().map_err(|e| ConfigError::InvalidValue("DB_MAX_CONNECTIONS".to_string(), e.to_string())))
            .transpose()?
            .unwrap_or(10);

        let grpc_bind_addr = optional_var("GRPC_BIND_ADDR")
            .unwrap_or_else(|| "0.0.0.0:50051".to_string());

        let health_bind_addr = optional_var("HEALTH_BIND_ADDR")
            .unwrap_or_else(|| "0.0.0.0:8080".to_string());

        Ok(Self {
            database_url,
            redis_url,
            broker_url,
            eventbus_url,
            db_max_connections,
            grpc_bind_addr,
            health_bind_addr,
        })
    }
}

fn required_var(name: &str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|_| ConfigError::MissingEnvVar(name.to_string()))
}

fn optional_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_database_url_returns_error() {
        // Only run if DATABASE_URL is not set in this environment
        if std::env::var("DATABASE_URL").is_err() {
            let result = RiskConfig::from_env();
            assert!(result.is_err());
            match result {
                Err(ConfigError::MissingEnvVar(name)) => assert_eq!(name, "DATABASE_URL"),
                _ => panic!("Expected MissingEnvVar(DATABASE_URL)"),
            }
        }
    }
}
