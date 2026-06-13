//! Event Bus configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Event Bus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Redis connection URL
    pub redis_url: String,

    /// Service name for identification
    pub service_name: String,

    /// Maximum events to keep per stream (approximate with Redis ~)
    pub stream_max_length: usize,

    /// Default consumer group settings
    pub consumer: ConsumerConfig,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Backpressure configuration
    pub backpressure: BackpressureConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Server configuration
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            service_name: "event-bus".to_string(),
            stream_max_length: 100_000,
            consumer: ConsumerConfig::default(),
            retry: RetryConfig::default(),
            backpressure: BackpressureConfig::default(),
            storage: StorageConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            service_name: std::env::var("SERVICE_NAME").unwrap_or_else(|_| "event-bus".to_string()),
            stream_max_length: std::env::var("STREAM_MAX_LENGTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100_000),
            consumer: ConsumerConfig::from_env(),
            retry: RetryConfig::from_env(),
            backpressure: BackpressureConfig::from_env(),
            storage: StorageConfig::from_env(),
            server: ServerConfig::from_env(),
        }
    }
}

/// Consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    /// Default batch size for reading
    pub batch_size: usize,

    /// Block time for XREADGROUP in milliseconds
    pub block_ms: usize,

    /// Max claims per pending check
    pub max_claims: usize,

    /// Minimum idle time to claim (ms)
    pub min_idle_time_ms: u64,

    /// Claim batch size
    pub claim_batch_size: usize,

    /// Max pending messages before pause
    pub max_pending: usize,

    /// Acknowledgment timeout
    pub ack_timeout_ms: u64,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            block_ms: 5000,
            max_claims: 100,
            min_idle_time_ms: 60_000, // 1 minute
            claim_batch_size: 10,
            max_pending: 10_000,
            ack_timeout_ms: 30_000,
        }
    }
}

impl ConsumerConfig {
    pub fn from_env() -> Self {
        let default = Self::default();
        Self {
            batch_size: env_parse("CONSUMER_BATCH_SIZE", default.batch_size),
            block_ms: env_parse("CONSUMER_BLOCK_MS", default.block_ms),
            max_claims: env_parse("CONSUMER_MAX_CLAIMS", default.max_claims),
            min_idle_time_ms: env_parse("CONSUMER_MIN_IDLE_MS", default.min_idle_time_ms),
            claim_batch_size: env_parse("CONSUMER_CLAIM_BATCH_SIZE", default.claim_batch_size),
            max_pending: env_parse("CONSUMER_MAX_PENDING", default.max_pending),
            ack_timeout_ms: env_parse("CONSUMER_ACK_TIMEOUT_MS", default.ack_timeout_ms),
        }
    }

    pub fn block_duration(&self) -> Duration {
        Duration::from_millis(self.block_ms as u64)
    }

    pub fn min_idle_duration(&self) -> Duration {
        Duration::from_millis(self.min_idle_time_ms)
    }

    pub fn ack_timeout(&self) -> Duration {
        Duration::from_millis(self.ack_timeout_ms)
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Max retry attempts
    pub max_retries: u32,

    /// Initial backoff duration
    pub initial_backoff_ms: u64,

    /// Maximum backoff duration
    pub max_backoff_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Jitter factor (0.0 - 1.0)
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 30_000,
            backoff_multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryConfig {
    pub fn from_env() -> Self {
        let default = Self::default();
        Self {
            max_retries: env_parse("RETRY_MAX_RETRIES", default.max_retries),
            initial_backoff_ms: env_parse("RETRY_INITIAL_BACKOFF_MS", default.initial_backoff_ms),
            max_backoff_ms: env_parse("RETRY_MAX_BACKOFF_MS", default.max_backoff_ms),
            backoff_multiplier: env_parse("RETRY_MULTIPLIER", default.backoff_multiplier),
            jitter: env_parse("RETRY_JITTER", default.jitter),
        }
    }
}

/// Backpressure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// Max number of events in buffer
    pub max_buffer_size: usize,

    /// Max pending publishes
    pub max_pending_publishes: usize,

    /// Pause publishing when buffer exceeds this
    pub pause_threshold: f64,

    /// Resume publishing when buffer drops below this
    pub resume_threshold: f64,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10_000,
            max_pending_publishes: 1_000,
            pause_threshold: 0.9,
            resume_threshold: 0.5,
        }
    }
}

impl BackpressureConfig {
    pub fn from_env() -> Self {
        let default = Self::default();
        Self {
            max_buffer_size: env_parse("BACKPRESSURE_MAX_BUFFER", default.max_buffer_size),
            max_pending_publishes: env_parse("BACKPRESSURE_MAX_PENDING", default.max_pending_publishes),
            pause_threshold: env_parse("BACKPRESSURE_PAUSE_THRESHOLD", default.pause_threshold),
            resume_threshold: env_parse("BACKPRESSURE_RESUME_THRESHOLD", default.resume_threshold),
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Enable persistent storage
    pub enabled: bool,

    /// PostgreSQL connection URL
    pub postgres_url: Option<String>,

    /// Max events to query at once
    pub query_limit: usize,

    /// Partition duration for events
    pub partition_duration_hours: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            postgres_url: None,
            query_limit: 1000,
            partition_duration_hours: 24,
        }
    }
}

impl StorageConfig {
    pub fn from_env() -> Self {
        let default = Self::default();
        Self {
            enabled: std::env::var("STORAGE_ENABLED")
                .map(|s| s.parse().unwrap_or(false))
                .unwrap_or(default.enabled),
            postgres_url: std::env::var("DATABASE_URL").ok(),
            query_limit: env_parse("STORAGE_QUERY_LIMIT", default.query_limit),
            partition_duration_hours: env_parse("STORAGE_PARTITION_HOURS", default.partition_duration_hours),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: String,

    /// Server port
    pub port: u16,

    /// Graceful shutdown timeout
    pub shutdown_timeout_secs: u64,

    /// Request timeout
    pub request_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            shutdown_timeout_secs: 30,
            request_timeout_secs: 30,
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let default = Self::default();
        Self {
            bind_address: std::env::var("BIND_ADDRESS").unwrap_or(default.bind_address),
            port: env_parse("SERVICE_PORT", default.port),
            shutdown_timeout_secs: env_parse("SHUTDOWN_TIMEOUT_SECS", default.shutdown_timeout_secs),
            request_timeout_secs: env_parse("REQUEST_TIMEOUT_SECS", default.request_timeout_secs),
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

/// Helper to parse environment variables with defaults
fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
