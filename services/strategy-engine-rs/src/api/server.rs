use axum::{routing::get, Router};
use std::net::SocketAddr;
use tonic::transport::Server as TonicServer;
use apex_protos::strategy::strategy_service_server::StrategyServiceServer;

use crate::api::service::{StrategyServiceImpl, StrategyState};
use crate::api::health::{health::liveness_check, readiness::readiness_check};
use crate::api::interceptors::{auth::auth_interceptor, metrics::MetricsLayer, logging::LoggingLayer};

/// Starts the multiplexed server (gRPC + HTTP health checks).
///
/// `state` is the live StrategyState initialised by the service bootstrap;
/// it is injected here so the gRPC layer never constructs its own state.
pub async fn start_server(
    grpc_addr: SocketAddr,
    http_addr: SocketAddr,
    state:     StrategyState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // ── HTTP Health Check Server ─────────────────────────────────────────────
    let app = Router::new()
        .route("/health", get(liveness_check))
        .route("/ready",  get(readiness_check));

    let axum_server = tokio::spawn(async move {
        let listener_result = tokio::net::TcpListener::bind(http_addr).await;
        if let Ok(listener) = listener_result {
            let _ = axum::serve(listener, app).await;
        }
    });

    // ── gRPC Server ──────────────────────────────────────────────────────────
    let grpc_server = tokio::spawn(async move {
        let strategy_service = StrategyServiceImpl::new(state);
        let svc = StrategyServiceServer::with_interceptor(strategy_service, auth_interceptor);

        let _ = TonicServer::builder()
            .layer(LoggingLayer)
            .layer(MetricsLayer)
            .add_service(svc)
            .serve(grpc_addr)
            .await;
    });

    let _ = tokio::try_join!(axum_server, grpc_server)?;
    Ok(())
}
