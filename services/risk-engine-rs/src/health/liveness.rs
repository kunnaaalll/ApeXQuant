use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct LivenessResponse {
    status: String,
}

pub async fn liveness_check() -> impl IntoResponse {
    Json(LivenessResponse {
        status: "alive".to_string(),
    })
}
