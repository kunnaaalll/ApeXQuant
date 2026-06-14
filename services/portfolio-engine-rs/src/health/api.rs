use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
    memory_usage_mb: f64,
    active_tasks: usize,
    timestamp: u64,
}

pub struct HealthState {
    pub active_tasks: AtomicUsize,
    // Add DB pools or other dependency handles here
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
            .unwrap()
            .as_secs(),
    })
}

async fn readiness(axum::extract::State(state): axum::extract::State<Arc<HealthState>>) -> Json<ReadinessResponse> {
    // In a real implementation, ping Postgres/Redis
    let postgres_connected = true;
    let redis_connected = true;

    // Determine status
    let status = if postgres_connected && redis_connected {
        "READY".to_string()
    } else {
        "UNAVAILABLE".to_string()
    };

    Json(ReadinessResponse {
        status,
        postgres_connected,
        redis_connected,
        memory_usage_mb: 0.0, // Placeholder
        active_tasks: state.active_tasks.load(Ordering::Relaxed),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}
