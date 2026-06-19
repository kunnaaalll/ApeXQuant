use axum::{routing::get, Router};
use std::net::SocketAddr;
use tonic::transport::Server as TonicServer;
use apex_protos::strategy::strategy_service_server::StrategyServiceServer;

use crate::api::service::StrategyServiceImpl;
use crate::api::health::{health::liveness_check, readiness::readiness_check};
use crate::api::interceptors::{auth::auth_interceptor, metrics::MetricsLayer, logging::LoggingLayer};

/// Starts the multiplexed server (gRPC and HTTP health checks on the same or separate ports).
/// For simplicity, we spawn two tasks: one for gRPC and one for HTTP health checks.
/// Fully deterministic and resilient.
pub async fn start_server(grpc_addr: SocketAddr, http_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // HTTP Health Check Server
    let app = Router::new()
        .route("/health", get(liveness_check))
        .route("/ready", get(readiness_check));

    let axum_server = tokio::spawn(async move {
        let listener_result = tokio::net::TcpListener::bind(http_addr).await;
        if let Ok(listener) = listener_result {
            let _ = axum::serve(listener, app).await;
        }
    });

    // gRPC Server
    let grpc_server = tokio::spawn(async move {
        let strategy_service = StrategyServiceImpl::default();
        let svc = StrategyServiceServer::with_interceptor(strategy_service, auth_interceptor);

        let _ = TonicServer::builder()
            .layer(LoggingLayer)
            .layer(MetricsLayer)
            .add_service(svc)
            .serve(grpc_addr)
            .await;
    });

    // Wait for both to complete (they typically run indefinitely)
    let _ = tokio::try_join!(axum_server, grpc_server)?;

    Ok(())
}
