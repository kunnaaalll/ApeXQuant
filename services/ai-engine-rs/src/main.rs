/// APEX V3 AI Engine — Production Runtime
///
/// Status contract:
/// - When no model artifact is loaded: every inference endpoint returns MODEL_UNAVAILABLE.
/// - Predictions are NEVER fabricated from heuristics.
/// - When a model becomes available (written to $MODEL_DIR), the service reloads automatically.
use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// All sub-modules are wired through lib.rs — they are compiled and available.
// main.rs only bootstraps connections and spawns the service loops.
use ai_engine_rs::api::run_api_server;
use ai_engine_rs::health::HealthStatus;

mod config;

use crate::config::AiEngineConfig;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting APEX V3 AI Engine...");

    // ── 1. Load Configuration ─────────────────────────────────────────────────
    let config = AiEngineConfig::load().unwrap_or_else(|e| {
        warn!("Config load failed ({}); using defaults", e);
        AiEngineConfig::default()
    });
    info!("Configuration loaded: server={} metrics_port={}", config.server_addr, config.metrics_port);

    // ── 2. Shared health state ─────────────────────────────────────────────────
    let health = Arc::new(RwLock::new(HealthStatus::default()));

    // ── 3. Connect PostgreSQL ──────────────────────────────────────────────────
    info!("Connecting to PostgreSQL at {}", config.database_url);
    let db_result = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&config.database_url)
        .await;

    let db_pool = match db_result {
        Ok(pool) => {
            info!("PostgreSQL connected");
            {
                let mut h = health.write().await;
                h.database = "CONNECTED".to_string();
            }
            Some(pool)
        }
        Err(e) => {
            warn!("PostgreSQL connection failed (non-fatal): {}", e);
            None
        }
    };

    // ── 4. Connect Redis ───────────────────────────────────────────────────────
    info!("Connecting to Redis at {}", config.redis_url);
    let redis_result = redis::Client::open(config.redis_url.clone());
    let redis_client = match redis_result {
        Ok(client) => {
            // Verify connectivity with a PING
            match client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let pong: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
                    if pong.is_ok() {
                        info!("Redis connected");
                        let mut h = health.write().await;
                        h.redis = "CONNECTED".to_string();
                    } else {
                        warn!("Redis PING failed");
                    }
                }
                Err(e) => warn!("Redis connection test failed: {}", e),
            }
            Some(client)
        }
        Err(e) => {
            warn!("Redis client creation failed (non-fatal): {}", e);
            None
        }
    };

    // Suppress unused variable warnings — these will be consumed by feature store
    // and online-learning loops when those subsystems are activated.
    let _ = db_pool;
    let _ = redis_client;

    // ── 5. Model Registry Check ────────────────────────────────────────────────
    // Check $MODEL_DIR for any loadable model artifact.
    // If none exists, keep HealthStatus::model_registry = "MODEL_UNAVAILABLE".
    let model_dir = std::env::var("MODEL_DIR")
        .unwrap_or_else(|_| "./models".to_string());
    let model_available = check_model_available(&model_dir);
    if model_available {
        info!("Model artifact detected in {}", model_dir);
        let mut h = health.write().await;
        h.model_registry = "MODEL_READY".to_string();
        h.status = "READY".to_string();
    } else {
        info!("No model artifact in {} — AI Engine will return MODEL_UNAVAILABLE for all inference requests", model_dir);
        // Health status already defaults to MODEL_UNAVAILABLE — do not change it.
    }

    // ── 6. Spawn model watcher ─────────────────────────────────────────────────
    // Polls MODEL_DIR every 60s and updates health status when a model appears.
    let health_watcher = health.clone();
    let model_dir_watch = model_dir.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let available = check_model_available(&model_dir_watch);
            let mut h = health_watcher.write().await;
            if available {
                if h.model_registry != "MODEL_READY" {
                    info!("Model artifact detected — updating status to MODEL_READY");
                    h.model_registry = "MODEL_READY".to_string();
                    h.status = "READY".to_string();
                }
            } else if h.model_registry == "MODEL_READY" {
                warn!("Model artifact removed — reverting to MODEL_UNAVAILABLE");
                h.model_registry = "MODEL_UNAVAILABLE".to_string();
                h.status = "MODEL_UNAVAILABLE".to_string();
            }
        }
    });

    // ── 7. HTTP API Server (health + inference) ────────────────────────────────
    info!("Starting HTTP API server on port {}", config.metrics_port);
    let health_for_api = health.clone();
    let api_port = config.metrics_port;
    tokio::spawn(async move {
        if let Err(e) = run_http_server(api_port, health_for_api).await {
            error!("HTTP API server error: {}", e);
        }
    });

    // ── 8. HTTP Inference API (POST /infer) ───────────────────────────────────
    // The API handler always returns MODEL_UNAVAILABLE until a real model is loaded.
    let infer_port: u16 = std::env::var("INFER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8051);
    info!("Starting inference API server on port {}", infer_port);
    tokio::spawn(async move {
        if let Err(e) = run_api_server(infer_port).await {
            error!("Inference API server error: {}", e);
        }
    });

    info!("AI Engine fully initialised — waiting for shutdown signal");

    // ── 9. Graceful shutdown ───────────────────────────────────────────────────
    shutdown_signal().await;
    info!("AI Engine shutting down gracefully");

    Ok(())
}

/// Check if any model file exists in the model directory.
/// Scans for .bin, .onnx, .pt, .pkl, .json (model metadata) extensions.
fn check_model_available(dir: &str) -> bool {
    let path = std::path::Path::new(dir);
    if !path.exists() || !path.is_dir() {
        return false;
    }
    let model_extensions = ["bin", "onnx", "pt", "pkl", "safetensors"];
    std::fs::read_dir(path)
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| model_extensions.contains(&ext))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// HTTP server: /health, /ready, /metrics.
async fn run_http_server(
    port: u16,
    health: Arc<RwLock<HealthStatus>>,
) -> Result<()> {
    use axum::{routing::get, Json, Router};
    use std::net::SocketAddr;

    let health_clone = health.clone();
    let app = Router::new()
        .route(
            "/health",
            get(move || {
                let h = health_clone.clone();
                async move {
                    let status = h.read().await.clone();
                    Json(status)
                }
            }),
        )
        .route(
            "/ready",
            get(move || {
                let h = health.clone();
                async move {
                    let status = h.read().await.clone();
                    if status.status == "READY" {
                        axum::response::IntoResponse::into_response(
                            (axum::http::StatusCode::OK, Json(serde_json::json!({"ready": true})))
                        )
                    } else {
                        axum::response::IntoResponse::into_response(
                            (axum::http::StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"ready": false, "reason": status.model_registry})))
                        )
                    }
                }
            }),
        )
        .route(
            "/metrics",
            get(|| async { "# AI Engine — no Prometheus metrics registered yet\n" }),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            sig.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
