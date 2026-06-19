use axum::{routing::get, Router};
use crate::health::{liveness::liveness_check, readiness::readiness_check};

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(liveness_check))
        .route("/ready", get(readiness_check))
}
