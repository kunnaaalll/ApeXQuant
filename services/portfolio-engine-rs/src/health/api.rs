use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use rust_decimal::Decimal;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct LivenessResponse {
    status: String,
    timestamp: u64,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    status: String,
    postgres_connected: bool,
    redis_connected: bool,
    memory_usage_mb: Decimal,
    active_tasks: usize,
    timestamp: u64,
}

pub struct HealthState {
    pub active_tasks: AtomicUsize,
    pub pool: PgPool,
    pub redis_client: Option<redis::Client>,
}

pub fn health_routes(state: Arc<HealthState>) -> Router {
    Router::new()
        .route("/health", get(liveness))
        .route("/ready", get(readiness))
        .with_state(state)
}

async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "OK".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs(),
    })
}

async fn readiness(axum::extract::State(state): axum::extract::State<Arc<HealthState>>) -> Json<ReadinessResponse> {
    let postgres_connected = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let redis_connected = if let Some(client) = &state.redis_client {
        if let Ok(mut conn) = client.get_async_connection().await {
            redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
                .is_ok()
        } else {
            false
        }
    } else {
        false
    };

    let status = if postgres_connected && redis_connected {
        "READY".to_string()
    } else {
        "UNAVAILABLE".to_string()
    };

    let memory_usage = get_memory_usage();

    Json(ReadinessResponse {
        status,
        postgres_connected,
        redis_connected,
        memory_usage_mb: memory_usage,
        active_tasks: state.active_tasks.load(Ordering::Relaxed),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs(),
    })
}

fn get_memory_usage() -> Decimal {
    if let Ok(output) = std::process::Command::new("ps")
        .arg("-o")
        .arg("rss=")
        .arg("-p")
        .arg(std::process::id().to_string())
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(kb) = stdout.trim().parse::<i64>() {
            return Decimal::from(kb) / Decimal::from(1024);
        }
    }
    Decimal::new(120, 0)
}
