use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub database_url: String,
    pub port: u16,
}

impl PerformanceConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/apex".to_string());
        let port = std::env::var("PERFORMANCE_ENGINE_PORT")
            .unwrap_or_else(|_| "50054".to_string())
            .parse()
            .unwrap_or(50054);

        Ok(Self { database_url, port })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub grpc_port: u16,
    pub eventbus_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            grpc_port: std::env::var("PERFORMANCE_ENGINE_GRPC_PORT")
                .unwrap_or_else(|_| "50054".to_string())
                .parse()
                .unwrap_or(50054),
            eventbus_url: std::env::var("EVENT_BUS_URL")
                .unwrap_or_else(|_| "http://localhost:50050".to_string()),
        }
    }
}
