use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::Instant;
use tracing::error;

/// Per-dependency health entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub name: String,
    pub healthy: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Aggregate health report for the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub service: String,
    pub ready: bool,      // true when all critical deps are healthy
    pub alive: bool,      // always true while process is running
    pub dependencies: Vec<DependencyHealth>,
    pub checked_at_ms: u64,
}

impl HealthReport {
    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Performs health checks against all runtime dependencies.
pub struct HealthChecker {
    service_name: String,
    db_pool: PgPool,
    redis_url: String,
    event_bus_url: String,
}

impl HealthChecker {
    pub fn new(
        service_name: String,
        db_pool: PgPool,
        redis_url: String,
        event_bus_url: String,
    ) -> Self {
        Self {
            service_name,
            db_pool,
            redis_url,
            event_bus_url,
        }
    }

    /// Run all dependency checks and return a full health report.
    pub async fn check(&self) -> HealthReport {
        let _start = Instant::now();
        let mut deps = Vec::new();

        deps.push(self.check_database().await);
        deps.push(self.check_redis().await);
        deps.push(self.check_event_bus().await);

        let ready = deps.iter().all(|d| d.healthy);

        HealthReport {
            service: self.service_name.clone(),
            ready,
            alive: true,
            dependencies: deps,
            checked_at_ms: HealthReport::now_ms(),
        }
    }

    async fn check_database(&self) -> DependencyHealth {
        let t = Instant::now();
        match tokio::time::timeout(
            Duration::from_secs(3),
            sqlx::query("SELECT 1").execute(&self.db_pool),
        )
        .await
        {
            Ok(Ok(_)) => DependencyHealth {
                name: "postgres".to_string(),
                healthy: true,
                latency_ms: t.elapsed().as_millis() as u64,
                error: None,
            },
            Ok(Err(e)) => {
                error!("Database health check failed: {}", e);
                DependencyHealth {
                    name: "postgres".to_string(),
                    healthy: false,
                    latency_ms: t.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                }
            }
            Err(_) => DependencyHealth {
                name: "postgres".to_string(),
                healthy: false,
                latency_ms: 3000,
                error: Some("Connection timeout after 3s".to_string()),
            },
        }
    }

    async fn check_redis(&self) -> DependencyHealth {
        let t = Instant::now();
        let result = tokio::time::timeout(Duration::from_secs(3), async {
            let client = redis::Client::open(self.redis_url.as_str())
                .map_err(|e| e.to_string())?;
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| e.to_string())?;
            redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
                .map_err(|e| e.to_string())
        })
        .await;

        match result {
            Ok(Ok(_)) => DependencyHealth {
                name: "redis".to_string(),
                healthy: true,
                latency_ms: t.elapsed().as_millis() as u64,
                error: None,
            },
            Ok(Err(e)) => DependencyHealth {
                name: "redis".to_string(),
                healthy: false,
                latency_ms: t.elapsed().as_millis() as u64,
                error: Some(e),
            },
            Err(_) => DependencyHealth {
                name: "redis".to_string(),
                healthy: false,
                latency_ms: 3000,
                error: Some("PING timeout after 3s".to_string()),
            },
        }
    }

    async fn check_event_bus(&self) -> DependencyHealth {
        let t = Instant::now();
        let url = self.event_bus_url.clone();
        let result = tokio::time::timeout(Duration::from_secs(3), async move {
            use apex_protos::events::event_bus_service_client::EventBusServiceClient;
            EventBusServiceClient::connect(url)
                .await
                .map_err(|e| e.to_string())
        })
        .await;

        match result {
            Ok(Ok(_)) => DependencyHealth {
                name: "event_bus".to_string(),
                healthy: true,
                latency_ms: t.elapsed().as_millis() as u64,
                error: None,
            },
            Ok(Err(e)) => DependencyHealth {
                name: "event_bus".to_string(),
                healthy: false,
                latency_ms: t.elapsed().as_millis() as u64,
                error: Some(e),
            },
            Err(_) => DependencyHealth {
                name: "event_bus".to_string(),
                healthy: false,
                latency_ms: 3000,
                error: Some("Connection timeout after 3s".to_string()),
            },
        }
    }
}
