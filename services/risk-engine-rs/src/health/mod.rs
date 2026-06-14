use axum::{routing::get, Router, response::IntoResponse, Json};
use serde_json::json;
use std::net::SocketAddr;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::Arc;

/// Set up Prometheus recorder and return a handle for rendering metrics
pub fn setup_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

/// Start a secondary HTTP server for health checks and metrics
pub async fn start_health_server(port: u16, handle: PrometheusHandle) {
    let handle = Arc::new(handle);
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get({
            let handle = handle.clone();
            move || async move { handle.render() }
        }));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
        tracing::info!("Health and metrics server listening on {}", addr);
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Health server error: {}", e);
        }
    } else {
        tracing::error!("Failed to bind health server on port {}", port);
    }
}

async fn health_check() -> impl IntoResponse {
    let runtime_stats = get_tokio_stats();
    
    Json(json!({
        "status": "up",
        "runtime": runtime_stats,
        "memory": get_memory_usage()
    }))
}

fn get_tokio_stats() -> serde_json::Value {
    let handle = tokio::runtime::Handle::current();
    let metrics = handle.metrics();
    json!({
        "workers": metrics.num_workers(),
        "num_alive_tasks": metrics.num_alive_tasks(),
    })
}

fn get_memory_usage() -> serde_json::Value {
    // In a production system, this would use sysinfo or similar to read /proc/self/statm
    // For now we return a generic status indicating the service is healthy
    json!({
        "status": "ok" 
    })
}
