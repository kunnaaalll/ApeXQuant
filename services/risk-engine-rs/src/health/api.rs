use crate::api::server::AppState;
use crate::health::{liveness::liveness_check, readiness::readiness_check};
use axum::{routing::get, Router};

pub fn health_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(liveness_check))
        .route("/ready", get(readiness_check))
        .with_state(state)
}
