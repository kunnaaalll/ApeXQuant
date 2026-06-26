#![deny(unsafe_code)]

//! Environment-driven configuration for execution-engine-rs.
//!
//! All fields load from environment variables with safe defaults.
//! No `unwrap()`, no `expect()`, no `panic!()`.

/// Complete runtime configuration loaded from the environment at startup.
#[derive(Debug, Clone)]
pub struct EnvironmentConfiguration {
    /// HTTP URL of the MT5 Manager API bridge sidecar.
    /// Default: `http://localhost:8001`
    pub mt5_bridge_url: String,

    /// Binance Futures REST base URL.
    /// Default: `https://testnet.binancefuture.com` (sandbox)
    pub binance_base_url: String,

    /// Binance API key (X-MBX-APIKEY header).
    pub binance_api_key: String,

    /// Binance API secret (HMAC-SHA256 signing key).
    pub binance_secret: String,

    /// gRPC server port. Default: 50052
    pub grpc_port: u16,

    /// HTTP health/readiness server port. Default: 8080
    pub http_port: u16,

    /// Event bus gRPC URL. Default: `http://localhost:50060`
    pub event_bus_url: String,

    /// PostgreSQL connection URL.
    pub postgres_url: String,

    /// Redis connection URL.
    pub redis_url: String,

    /// Heartbeat loop interval in seconds. Default: 30
    pub heartbeat_interval_secs: u64,

    /// Health check loop interval in seconds. Default: 60
    pub health_check_interval_secs: u64,

    /// Reconciliation loop interval in seconds. Default: 120
    pub reconciliation_interval_secs: u64,
}

impl EnvironmentConfiguration {
    /// Load configuration from environment variables.
    ///
    /// Returns a fully-populated config with safe defaults for any
    /// missing variables. Never panics, never unwraps.
    pub fn from_env() -> Self {
        Self {
            mt5_bridge_url: env_or("MT5_BRIDGE_URL", "http://localhost:8001"),
            binance_base_url: env_or(
                "BINANCE_BASE_URL",
                "https://testnet.binancefuture.com",
            ),
            binance_api_key: env_or("BINANCE_API_KEY", ""),
            binance_secret: env_or("BINANCE_SECRET", ""),
            grpc_port: env_u16("GRPC_PORT", 50052),
            http_port: env_u16("HTTP_PORT", 8080),
            event_bus_url: env_or("EVENT_BUS_URL", "http://localhost:50060"),
            postgres_url: env_or(
                "DATABASE_URL",
                "postgres://apex:apex@localhost:5432/apex_execution",
            ),
            redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
            heartbeat_interval_secs: env_u64("HEARTBEAT_INTERVAL_SECS", 30),
            health_check_interval_secs: env_u64("HEALTH_CHECK_INTERVAL_SECS", 60),
            reconciliation_interval_secs: env_u64("RECONCILIATION_INTERVAL_SECS", 120),
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_load_without_env() {
        // Ensure from_env() never panics even with no env vars set.
        let cfg = EnvironmentConfiguration::from_env();
        assert!(!cfg.mt5_bridge_url.is_empty());
        assert!(!cfg.binance_base_url.is_empty());
        assert!(cfg.grpc_port > 0);
        assert!(cfg.http_port > 0);
    }

    #[test]
    fn test_env_u16_parse_failure_falls_back() {
        // Feed a bad value — must not panic, must use default.
        std::env::set_var("TEST_U16_PORT", "not_a_number");
        let port = env_u16("TEST_U16_PORT", 9999);
        std::env::remove_var("TEST_U16_PORT");
        assert_eq!(port, 9999);
    }
}
