use axum::{response::IntoResponse, Json, http::StatusCode, extract::State};
use serde::Serialize;
use crate::api::server::AppState;

#[derive(Serialize)]
pub struct ReadinessResponse {
    status: String,
    postgres: String,
    redis: String,
}

pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut pg_status = "ok".to_string();
    if let Err(e) = sqlx::query("SELECT 1").execute(&state.pg_pool).await {
        pg_status = format!("error: {}", e);
    }

    let mut redis_status = "ok".to_string();
    let conn_result = state.redis_client.get_multiplexed_async_connection().await;
    match conn_result {
        Ok(mut conn) => {
            if let Err(e) = redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                redis_status = format!("error: {}", e);
            }
        }
        Err(e) => {
            redis_status = format!("error: {}", e);
        }
    }

    let overall = if pg_status == "ok" && redis_status == "ok" {
        "ready"
    } else {
        "unhealthy"
    };

    let response = ReadinessResponse {
        status: overall.to_string(),
        postgres: pg_status,
        redis: redis_status,
    };

    if overall == "ready" {
        (StatusCode::OK, Json(response))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

