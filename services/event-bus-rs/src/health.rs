//! Health check endpoints

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatus {
    pub status: HealthState,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub components: Vec<ComponentHealth>,
}

/// Individual component state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthState {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthState::Healthy)
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthState,
    pub message: Option<String>,
    pub latency_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Check health of all components
pub async fn check_health(redis: &ConnectionManager) -> HealthStatus {
    let start = std::time::Instant::now();
    let timestamp = chrono::Utc::now();

    let mut components = vec![];

    // Check Redis
    let redis_health = check_redis(redis).await;
    components.push(redis_health);

    // Overall status is worst of components
    let status = if components.iter().all(|c| c.status == HealthState::Healthy) {
        HealthState::Healthy
    } else if components.iter().any(|c| c.status == HealthState::Unhealthy) {
        HealthState::Unhealthy
    } else {
        HealthState::Degraded
    };

    HealthStatus {
        status,
        timestamp,
        version: env!("CARGO_PKG_VERSION").to_string(),
        components,
    }
}

/// Check Redis connectivity
async fn check_redis(redis: &ConnectionManager) -> ComponentHealth {
    let start = std::time::Instant::now();
    let now = chrono::Utc::now();

    match redis::cmd("PING").query_async::<_, String>(redis).await {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;

            if response == "PONG" {
                ComponentHealth {
                    name: "redis".to_string(),
                    status: HealthState::Healthy,
                    message: Some("Connected".to_string()),
                    latency_ms: latency,
                    last_check: now,
                }
            } else {
                ComponentHealth {
                    name: "redis".to_string(),
                    status: HealthState::Degraded,
                    message: Some(format!("Unexpected response: {}", response)),
                    latency_ms: latency,
                    last_check: now,
                }
            }
        }
        Err(e) => ComponentHealth {
            name: "redis".to_string(),
            status: HealthState::Unhealthy,
            message: Some(format!("Connection failed: {}", e)),
            latency_ms: start.elapsed().as_millis() as u64,
            last_check: now,
        },
    }
}

/// Readiness probe - can this instance receive traffic?
pub async fn is_ready(bus: &crate::EventBus) -> bool {
    let health = bus.health().await;
    matches!(health.status, HealthState::Healthy | HealthState::Degraded)
}

/// Liveness probe - is this instance alive?
pub async fn is_alive(redis: &ConnectionManager) -> bool {
    let result: redis::RedisResult<String> =
        redis::cmd("PING").query_async(redis).await;
    result.is_ok()
}

/// Startup probe - has this instance started?
pub async fn has_started(redis: &ConnectionManager, timeout: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + timeout;

    while tokio::time::Instant::now() < deadline {
        if is_alive(redis).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_state() {
        assert!(HealthState::Healthy.is_healthy());
        assert!(!HealthState::Degraded.is_healthy());
        assert!(!HealthState::Unhealthy.is_healthy());
    }

    // Integration tests would require Redis
}
