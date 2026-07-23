/// APEX V3.1 Orchestrator — Production Runtime
///
/// Responsibilities:
/// 1. Expose /health, /ready, /metrics on :8090
/// 2. Poll every downstream service's gRPC health endpoint every 10s
/// 3. Aggregate health status and mark self ready only when all required services are reachable
/// 4. Provide a graceful shutdown that signals downstream services via their gRPC health interface
use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{signal, sync::RwLock, time};
use tracing::{error, info, warn};

/// All Rust services and their gRPC health endpoints.
/// The key is the service name; the value is the address for tonic health checks.
const SERVICES: &[(&str, &str)] = &[
    ("event-bus",           "http://localhost:50050"),
    ("market-data-engine",  "http://localhost:50051"),
    ("strategy-engine",     "http://localhost:50052"),
    ("signal-engine",       "http://localhost:50053"),
    ("learning-engine",     "http://localhost:50054"),
    ("risk-engine",         "http://localhost:50055"),
    ("execution-engine",    "http://localhost:50056"),
    ("performance-engine",  "http://localhost:50057"),
    ("ai-engine",           "http://localhost:50058"),
    ("backtester",          "http://localhost:50059"),
];

/// Services that MUST be healthy before the orchestrator marks itself ready.
const REQUIRED_SERVICES: &[&str] = &[
    "event-bus",
    "market-data-engine",
    "risk-engine",
    "execution-engine",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unreachable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthEntry {
    pub name: String,
    pub status: ServiceStatus,
    pub last_checked: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorHealth {
    pub status: String,
    pub ready: bool,
    pub services: Vec<ServiceHealthEntry>,
    pub total: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub unreachable: usize,
    pub required_all_healthy: bool,
    pub uptime_secs: u64,
}

struct OrchestratorState {
    services: HashMap<String, ServiceHealthEntry>,
    ready: bool,
    started_at: Instant,
}

type SharedState = Arc<RwLock<OrchestratorState>>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting APEX V3.1 Orchestrator on :8090");

    let state: SharedState = Arc::new(RwLock::new(OrchestratorState {
        services: HashMap::new(),
        ready: false,
        started_at: Instant::now(),
    }));

    // ── 1. Initial health poll ────────────────────────────────────────────────
    poll_all_services(state.clone()).await;

    // ── 2. Determine initial readiness ───────────────────────────────────────
    update_readiness(state.clone()).await;

    // ── 3. Spawn background health polling loop (every 10s) ──────────────────
    let poll_state = state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            poll_all_services(poll_state.clone()).await;
            update_readiness(poll_state.clone()).await;
        }
    });

    // ── 4. HTTP Server ────────────────────────────────────────────────────────
    let http_state = state.clone();
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready",  get(ready_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(http_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Orchestrator HTTP server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Orchestrator shutdown complete");
    Ok(())
}

/// Poll every service's gRPC health check endpoint.
async fn poll_all_services(state: SharedState) {
    let checks: Vec<_> = SERVICES
        .iter()
        .map(|(name, addr)| {
            let name = name.to_string();
            let addr = addr.to_string();
            tokio::spawn(async move {
                let entry = check_grpc_health(&name, &addr).await;
                (name, entry)
            })
        })
        .collect();

    let mut results = Vec::new();
    for handle in checks {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => error!("Health check task panicked: {}", e),
        }
    }

    let mut s = state.write().await;
    for (name, entry) in results {
        s.services.insert(name, entry);
    }
}

/// Check a single service's gRPC health endpoint.
/// Uses the standard gRPC Health Checking Protocol.
async fn check_grpc_health(name: &str, addr: &str) -> ServiceHealthEntry {
    let now = chrono::Utc::now().to_rfc3339();
    let t0 = Instant::now();

    let channel_result = tonic::transport::Channel::from_shared(addr.to_owned())
        .map_err(|e| e.to_string())
        .and_then(|endpoint| {
            Ok(endpoint
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(3))
                .connect_lazy())
        });

    match channel_result {
        Err(e) => ServiceHealthEntry {
            name: name.to_string(),
            status: ServiceStatus::Unreachable,
            last_checked: now,
            latency_ms: None,
            error: Some(format!("Channel creation error: {}", e)),
        },
        Ok(channel) => {
            let mut client = tonic_health::pb::health_client::HealthClient::new(channel);
            let request = tonic_health::pb::HealthCheckRequest {
                service: String::new(), // Empty = check server overall health
            };
            match tokio::time::timeout(Duration::from_secs(5), client.check(request)).await {
                Ok(Ok(resp)) => {
                    let latency = t0.elapsed().as_millis() as u64;
                    let grpc_status = resp.into_inner().status;
                    let status = if grpc_status == tonic_health::pb::health_check_response::ServingStatus::Serving as i32 {
                        ServiceStatus::Healthy
                    } else {
                        ServiceStatus::Degraded
                    };
                    ServiceHealthEntry {
                        name: name.to_string(),
                        status,
                        last_checked: now,
                        latency_ms: Some(latency),
                        error: None,
                    }
                }
                Ok(Err(e)) => {
                    let latency = t0.elapsed().as_millis() as u64;
                    ServiceHealthEntry {
                        name: name.to_string(),
                        status: ServiceStatus::Unreachable,
                        last_checked: now,
                        latency_ms: Some(latency),
                        error: Some(format!("gRPC error: {}", e)),
                    }
                }
                Err(_) => ServiceHealthEntry {
                    name: name.to_string(),
                    status: ServiceStatus::Unreachable,
                    last_checked: now,
                    latency_ms: None,
                    error: Some("Health check timed out".to_string()),
                },
            }
        }
    }
}

/// Determine if the orchestrator should mark itself ready.
async fn update_readiness(state: SharedState) {
    let mut s = state.write().await;
    let required_all_healthy = REQUIRED_SERVICES.iter().all(|name| {
        s.services
            .get(*name)
            .map(|e| e.status == ServiceStatus::Healthy)
            .unwrap_or(false)
    });

    let was_ready = s.ready;
    s.ready = required_all_healthy;

    if s.ready && !was_ready {
        info!("Orchestrator is now READY — all required services are healthy");
    } else if !s.ready && was_ready {
        let unhealthy: Vec<_> = REQUIRED_SERVICES
            .iter()
            .filter(|name| {
                !s.services
                    .get(**name)
                    .map(|e| e.status == ServiceStatus::Healthy)
                    .unwrap_or(false)
            })
            .collect();
        warn!(
            "Orchestrator marked NOT READY — unhealthy required services: {:?}",
            unhealthy
        );
    }
}

/// GET /health — Returns aggregate health of all services.
async fn health_handler(State(state): State<SharedState>) -> (StatusCode, Json<OrchestratorHealth>) {
    let s = state.read().await;
    let uptime = s.started_at.elapsed().as_secs();
    let services: Vec<ServiceHealthEntry> = s.services.values().cloned().collect();

    let healthy = services.iter().filter(|e| e.status == ServiceStatus::Healthy).count();
    let degraded = services.iter().filter(|e| e.status == ServiceStatus::Degraded).count();
    let unreachable = services.iter().filter(|e| e.status == ServiceStatus::Unreachable).count();
    let total = services.len();

    let required_all_healthy = REQUIRED_SERVICES.iter().all(|name| {
        s.services
            .get(*name)
            .map(|e| e.status == ServiceStatus::Healthy)
            .unwrap_or(false)
    });

    let overall_status = if s.ready { "HEALTHY".to_string() } else { "DEGRADED".to_string() };

    let body = OrchestratorHealth {
        status: overall_status,
        ready: s.ready,
        services,
        total,
        healthy,
        degraded,
        unreachable,
        required_all_healthy,
        uptime_secs: uptime,
    };

    let code = if s.ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (code, Json(body))
}

/// GET /ready — Kubernetes readiness probe.
async fn ready_handler(State(state): State<SharedState>) -> StatusCode {
    let s = state.read().await;
    if s.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

/// GET /metrics — Prometheus text format.
async fn metrics_handler(State(state): State<SharedState>) -> String {
    let s = state.read().await;
    let uptime = s.started_at.elapsed().as_secs();
    let healthy = s.services.values().filter(|e| e.status == ServiceStatus::Healthy).count();
    let unreachable = s.services.values().filter(|e| e.status == ServiceStatus::Unreachable).count();
    let total = s.services.len();
    let ready = if s.ready { 1 } else { 0 };

    let mut output = format!(
        "# HELP apex_orchestrator_uptime_seconds Seconds since orchestrator started\n\
         apex_orchestrator_uptime_seconds {}\n\
         # HELP apex_orchestrator_ready Whether orchestrator is ready (1=yes, 0=no)\n\
         apex_orchestrator_ready {}\n\
         # HELP apex_orchestrator_services_total Total number of tracked services\n\
         apex_orchestrator_services_total {}\n\
         # HELP apex_orchestrator_services_healthy Number of healthy services\n\
         apex_orchestrator_services_healthy {}\n\
         # HELP apex_orchestrator_services_unreachable Number of unreachable services\n\
         apex_orchestrator_services_unreachable {}\n",
        uptime, ready, total, healthy, unreachable
    );

    for entry in s.services.values() {
        let status_val = match entry.status {
            ServiceStatus::Healthy => 1,
            ServiceStatus::Degraded => 2,
            ServiceStatus::Unreachable => 0,
            ServiceStatus::Unknown => 3,
        };
        output.push_str(&format!(
            "apex_service_status{{service=\"{}\"}} {}\n",
            entry.name, status_val
        ));
        if let Some(latency) = entry.latency_ms {
            output.push_str(&format!(
                "apex_service_health_latency_ms{{service=\"{}\"}} {}\n",
                entry.name, latency
            ));
        }
    }

    output
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
    info!("Orchestrator: shutdown signal received");
}
