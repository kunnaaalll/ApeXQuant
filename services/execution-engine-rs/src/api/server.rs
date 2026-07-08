use apex_protos::execution::execution_service_server::ExecutionServiceServer;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tonic::transport::Server as TonicServer;

use crate::api::auth::auth_interceptor;
use crate::api::health::health_handler;
use crate::api::middleware::build_middleware_stack;
use crate::api::readiness::readiness_handler;
use crate::api::service::ExecutionServiceImpl;
use crate::event_bus::EventBusPublisher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pg_store: Option<Arc<crate::storage::pg_store::PgStore>>,
    pub redis_client: Option<redis::Client>,
    pub mt5_adapter: Arc<crate::brokers::mt5::adapter::Mt5Adapter>,
    pub binance_adapter: Arc<crate::brokers::binance::adapter::BinanceAdapter>,
}

pub async fn start_api_servers(
    grpc_port: u16,
    http_port: u16,
    event_bus: Option<Arc<EventBusPublisher>>,
    mt5_adapter: Arc<crate::brokers::mt5::adapter::Mt5Adapter>,
    binance_adapter: Arc<crate::brokers::binance::adapter::BinanceAdapter>,
    pg_store: Option<Arc<crate::storage::pg_store::PgStore>>,
    redis_client: Option<redis::Client>,
) -> (
    JoinHandle<Result<(), tonic::transport::Error>>,
    JoinHandle<Result<(), std::io::Error>>,
) {
    let state = AppState {
        pg_store: pg_store.clone(),
        redis_client,
        mt5_adapter: mt5_adapter.clone(),
        binance_adapter: binance_adapter.clone(),
    };

    let axum_app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .with_state(state);

    let http_addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let axum_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await?;
        axum::serve(listener, axum_app).await
    });

    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], grpc_port));
    let execution_service = ExecutionServiceServer::with_interceptor(
        ExecutionServiceImpl::new(event_bus, mt5_adapter, binance_adapter, pg_store),
        auth_interceptor,
    );

    let grpc_handle = tokio::spawn(async move {
        TonicServer::builder()
            .layer(build_middleware_stack())
            .add_service(execution_service)
            .serve(grpc_addr)
            .await
    });

    (grpc_handle, axum_handle)
}
