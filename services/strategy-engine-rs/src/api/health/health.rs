use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Simple liveness probe returning 200 OK.
/// Does not check dependencies.
pub async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, "Alive")
}
