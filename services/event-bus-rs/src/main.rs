//! APEX V3 Event Bus Service
//!
//! Entry point for the event bus microservice.

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing::{info, warn};

pub mod config;
pub mod error;
pub mod event;
pub mod health;
pub mod metrics;
pub mod publisher;
pub mod server;
pub mod storage;
pub mod subscriber;
pub mod subscription;

use config::Config;

/// Main entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_logging();

    info!(
        service = "event-bus",
        version = env!("CARGO_PKG_VERSION"),
        "Starting APEX V3 Event Bus"
    );

    // Load configuration
    let config = Config::from_env();
    info!(config = ?config, "Configuration loaded");

    // Initialize event bus
    let event_bus = Arc::new(init_event_bus(config.clone()).await?);
    info!("Event Bus initialized");

    // Create HTTP server
    let app = create_routes(event_bus.clone());
    let addr = config.server.bind_addr();

    info!(addr = %addr, "Starting HTTP server");

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Event Bus shutdown complete");
    Ok(())
}

/// Initialize logging
fn init_logging() {
    use tracing_subscriber::prelude::*;

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Initialize the event bus
async fn init_event_bus(config: Config) -> Result<event_bus_rs::EventBus, error::EventBusError> {
    event_bus_rs::EventBus::new(config).await
}

/// Create HTTP routes
fn create_routes(event_bus: Arc<event_bus_rs::EventBus>) -> Router {
    use axum::extract::State;
    use axum::response::Json;
    use serde_json::json;

    Router::new()
        .route("/health", get(health_handler))
        .route("/readiness", get(readiness_handler))
        .route("/liveness", get(liveness_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(event_bus)
}

/// Health check handler
async fn health_handler(
    State(bus): State<Arc<event_bus_rs::EventBus>>,
) -> axum::response::Response<axum::body::Body> {
    let health = bus.health().await;
    let status_code = match health.status {
        health::HealthState::Healthy => axum::http::StatusCode::OK,
        health::HealthState::Degraded => axum::http::StatusCode::OK,
        health::HealthState::Unhealthy => axum::http::StatusCode::SERVICE_UNAVAILABLE,
    };

    let body = axum::body::Body::from(serde_json::to_string(&health).unwrap());

    axum::response::Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}

/// Readiness probe handler
async fn readiness_handler(
    State(bus): State<Arc<event_bus_rs::EventBus>>,
) -> axum::http::StatusCode {
    if health::is_ready(&bus).await {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    }
}

/// Liveness probe handler
async fn liveness_handler(
    State(bus): State<Arc<event_bus_rs::EventBus>>,
) -> axum::http::StatusCode {
    // Check if alive (simplified - would check redis directly)
    axum::http::StatusCode::OK
}

/// Prometheus metrics handler
async fn metrics_handler(
    State(bus): State<Arc<event_bus_rs::EventBus>>,
) -> axum::response::Response<axum::body::Body> {
    let metrics = bus.metrics().export();

    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(axum::body::Body::from(metrics))
        .unwrap()
}

/// Shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down"),
        _ = terminate => info!("Received SIGTERM, shutting down"),
    }
}

// Stub implementations for compilation
#[allow(dead_code)]
mod event_bus_rs {
    pub use crate::config::Config;
    pub use crate::error::EventBusError;
    pub use crate::event::{CorrelationContext, Event, EventMetadata, EventPayload};
    pub use crate::health::{self, ComponentHealth, HealthState, HealthStatus};
    pub use crate::metrics::EventBusMetrics;
    pub use crate::publisher::EventPublisher;
    pub use crate::subscriber::EventSubscriber;

    use crate::storage::EventStorage;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Clone)]
    pub struct EventBus {
        config: Arc<Config>,
        redis: redis::aio::ConnectionManager,
        publisher: EventPublisher,
        storage: Arc<RwLock<EventStorage>>,
        metrics: EventBusMetrics,
    }

    impl EventBus {
        pub async fn new(config: Config) -> Result<Self, EventBusError> {
            let redis_client = redis::Client::open(config.redis_url.clone())?;
            let redis = redis::aio::ConnectionManager::new(redis_client).await?;

            Ok(Self {
                config: Arc::new(config),
                redis: redis.clone(),
                publisher: EventPublisher::new(redis.clone()),
                storage: Arc::new(RwLock::new(EventStorage::new(redis, &config))),
                metrics: EventBusMetrics::new(),
            })
        }

        pub async fn publish(&self, _event: Event) -> crate::Result<String> {
            Ok("test-id".to_string())
        }

        pub async fn health(&self) -> HealthStatus {
            crate::health::check_health(&self.redis).await
        }

        pub fn metrics(&self) -> &EventBusMetrics {
            &self.metrics
        }
    }
}
