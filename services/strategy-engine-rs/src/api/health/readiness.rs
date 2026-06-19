use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Readiness probe.
/// In a full implementation, this checks connections to PostgreSQL and Redis.
/// For the pure transport layer implementation, we simulate a successful check
/// as the engines aren't wired up yet.
pub async fn readiness_check() -> impl IntoResponse {
    let pg_ready = true; // Simulated check
    let redis_ready = true; // Simulated check

    if pg_ready && redis_ready {
        (StatusCode::OK, "Ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "NotReady")
    }
}
