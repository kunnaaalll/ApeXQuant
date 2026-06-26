use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tonic::transport::Server as TonicServer;
use apex_protos::execution::execution_service_server::ExecutionServiceServer;

use crate::api::service::ExecutionServiceImpl;
use crate::event_bus::EventBusPublisher;
use std::sync::Arc;
use crate::api::health::health_handler;
use crate::api::readiness::readiness_handler;
use crate::api::middleware::build_middleware_stack;
use crate::api::auth::auth_interceptor;

pub async fn start_api_servers(grpc_port: u16, http_port: u16, event_bus: Option<Arc<EventBusPublisher>>) -> (JoinHandle<Result<(), tonic::transport::Error>>, JoinHandle<Result<(), std::io::Error>>) {
    let axum_app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler));

    let http_addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let axum_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await?;
        axum::serve(listener, axum_app).await
    });

    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], grpc_port));
    let execution_service = ExecutionServiceServer::with_interceptor(ExecutionServiceImpl::new(event_bus), auth_interceptor);
    
    let grpc_handle = tokio::spawn(async move {
        TonicServer::builder()
            .layer(build_middleware_stack())
            .add_service(execution_service)
            .serve(grpc_addr)
            .await
    });

    (grpc_handle, axum_handle)
}
