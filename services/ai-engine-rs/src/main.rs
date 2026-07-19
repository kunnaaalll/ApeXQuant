use anyhow::Result;
use tokio::signal;
use tracing::{info, warn};

mod config;
mod health;
mod metrics;

// These will be implemented later, for now just declare the modules
// mod feature_engineering;
// mod regime_classifier;
// mod pattern_recognition;
// mod reinforcement;
// mod training;
// mod online_learning;
// mod explainability;
// mod feedback;
// mod embeddings;
// mod vector_store;
// mod dataset;
// mod grpc;
// mod api;

use crate::config::AiEngineConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::fmt::init();
    info!("Starting AI Engine...");

    // Load configuration
    let config = AiEngineConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config from env, using defaults: {}", e);
        AiEngineConfig::default()
    });

    info!("Connecting to PostgreSQL at {}", config.database_url);
    // let db_pool = sqlx::postgres::PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&config.database_url)
    //     .await?;

    info!("Connecting to Redis at {}", config.redis_url);
    // let redis_client = redis::Client::open(config.redis_url.clone())?;
    // let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;

    info!("Initializing Event Bus (Kafka) at {}", config.event_bus_url);

    info!("Initializing Feature Store...");
    info!("Initializing Model Registry...");
    info!("Initializing Background Workers...");

    info!("Starting gRPC server on {}", config.server_addr);
    // tokio::spawn(async move {
    //     // Start gRPC
    // });

    let metrics_registry = std::sync::Arc::new(crate::metrics::MetricsRegistry::new());
    let health_status = std::sync::Arc::new(tokio::sync::RwLock::new(
        crate::health::HealthStatus::default(),
    ));

    info!(
        "Starting Metrics and Health server on port {}",
        config.metrics_port
    );
    let metrics_registry_clone = metrics_registry.clone();
    let health_status_clone = health_status.clone();
    let app = axum::Router::new()
        .route(
            "/health",
            axum::routing::get(move || {
                let health = health_status_clone.clone();
                async move {
                    let status = health.read().await.clone();
                    axum::Json(status)
                }
            }),
        )
        .route(
            "/metrics",
            axum::routing::get(move || {
                let metrics = metrics_registry_clone.clone();
                async move { metrics.gather() }
            }),
        );

    let metrics_addr = format!("0.0.0.0:{}", config.metrics_port);
    let listener = tokio::net::TcpListener::bind(&metrics_addr).await?;
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Metrics server error: {}", e);
        }
    });

    // Wait for graceful shutdown
    shutdown_signal().await;
    info!("Shutting down AI Engine gracefully.");

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
