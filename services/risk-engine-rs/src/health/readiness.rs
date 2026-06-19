use axum::{response::IntoResponse, Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ReadinessResponse {
    status: String,
}

pub async fn readiness_check() -> impl IntoResponse {
    // In a real app we would check pg_pool, redis, etc.
    // For this transport layer mockup, we return ready.
    (
        StatusCode::OK,
        Json(ReadinessResponse {
            status: "ready".to_string(),
        }),
    )
}
