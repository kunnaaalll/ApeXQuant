pub mod service;

use std::net::SocketAddr;
use apex_protos::analytics::analytics_engine_server::AnalyticsEngineServer;
use tonic::transport::Server;
use std::sync::Arc;
use axum::{routing::get, Router};
use metrics_exporter_prometheus::PrometheusBuilder;

use crate::storage::Repositories;
use crate::analytics::engine::AnalyticsEngine;

pub async fn start_api_server(
    grpc_addr: SocketAddr,
    http_addr: SocketAddr,
    repos: Arc<Repositories>,
    engine: Arc<AnalyticsEngine>,
) -> anyhow::Result<()> {
    tracing::info!("Starting Performance Engine gRPC server on {}", grpc_addr);
    tracing::info!("Starting Performance Engine HTTP server on {}", http_addr);
    
    // 1. Setup Prometheus metrics exporter
    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
        
    // 2. Setup HTTP Server (Metrics & Health)
    let app = Router::new()
        .route("/health/liveness", get(|| async { "OK" }))
        .route("/health/readiness", get({
            let repos = repos.clone();
            move || async move {
                if repos.performance.health_check().await {
                    (axum::http::StatusCode::OK, "Ready")
                } else {
                    (axum::http::StatusCode::SERVICE_UNAVAILABLE, "Not Ready")
                }
            }
        }))
        .route("/metrics", get(move || std::future::ready(recorder_handle.render())));

    let http_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // 3. Setup gRPC Server
    let analytics_service = service::AnalyticsServiceImpl::new(repos, engine);

    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(AnalyticsEngineServer::new(analytics_service))
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    // Wait for either server to exit (which shouldn't happen normally unless killed)
    tokio::select! {
        _ = http_task => { tracing::error!("HTTP server exited unexpectedly"); },
        _ = grpc_task => { tracing::error!("gRPC server exited unexpectedly"); },
    };

    Ok(())
}
