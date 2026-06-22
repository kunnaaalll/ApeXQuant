use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
}

pub async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "alive".to_string(),
    })
}
