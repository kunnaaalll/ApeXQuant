use crate::api::server::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
}

pub async fn health_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let mut status = "alive".to_string();
    if let Some(pg) = &app_state.pg_store {
        if sqlx::query("SELECT 1").execute(pg.pool()).await.is_err() {
            status = "degraded (database down)".to_string();
        }
    }

    Json(HealthResponse { status })
}
