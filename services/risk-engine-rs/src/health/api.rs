use axum::{routing::get, Router};
use crate::health::{liveness::liveness_check, readiness::readiness_check};
use crate::api::server::AppState;

pub fn health_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(liveness_check))
        .route("/ready", get(readiness_check))
        .with_state(state)
}
