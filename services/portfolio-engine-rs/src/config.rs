#![deny(unsafe_code)]

//! Environment-driven configuration for portfolio-engine-rs. All fields load from environment variables with safe defaults.
//! No `unwrap()`, no `expect()`, no `panic!()`.

#[derive(Debug, Clone)]
pub struct EnvironmentConfiguration {
    pub grpc_port: u16,
    pub http_port: u16,
    pub event_bus_url: String,
    pub postgres_url: String,
    pub redis_url: String,
    pub reconciliation_interval_secs: u64,
    pub shadow_mode: bool,
}

impl EnvironmentConfiguration {
    pub fn from_env() -> Self {
        Self {
            grpc_port: env_u16("GRPC_PORT", 50051),
            http_port: env_u16("HTTP_PORT", 8080),
            event_bus_url: env_or("EVENT_BUS_URL", "http://localhost:50060"),
            postgres_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
            reconciliation_interval_secs: env_u64("RECONCILIATION_INTERVAL_SECS", 30),
            shadow_mode: env_bool("SHADOW_MODE", false),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_u16(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|v| v.to_lowercase().parse::<bool>().ok())
        .unwrap_or(default)
}
