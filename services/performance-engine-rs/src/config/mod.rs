use serde::{Deserialize, Serialize};

/// Unified production configuration for the Performance Engine.
/// All values are sourced from environment variables with sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// PostgreSQL connection URL
    pub database_url: String,
    /// Redis connection URL
    pub redis_url: String,
    /// gRPC Event Bus URL (the APEX event-bus-rs service)
    pub event_bus_url: String,
    /// gRPC port for the Analytics Engine service
    pub grpc_port: u16,
    /// HTTP port for Prometheus metrics and health endpoints
    pub http_port: u16,
    /// Maximum PostgreSQL pool connections
    pub db_pool_max: u32,
    /// Service name used in tracing and health checks
    pub service_name: String,
    /// Consumer group name for event bus subscriptions
    pub consumer_group: String,
}

impl AppConfig {
    /// Load configuration from environment variables with production-safe defaults.
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
            }),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            event_bus_url: std::env::var("EVENT_BUS_URL")
                .unwrap_or_else(|_| "http://localhost:50050".to_string()),
            grpc_port: std::env::var("PERFORMANCE_ENGINE_GRPC_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50054),
            http_port: std::env::var("PERFORMANCE_ENGINE_HTTP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9090),
            db_pool_max: std::env::var("DB_POOL_MAX")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            service_name: std::env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "performance-engine".to_string()),
            consumer_group: std::env::var("EVENT_BUS_CONSUMER_GROUP")
                .unwrap_or_else(|_| "performance_engine_group".to_string()),
        }
    }
}
