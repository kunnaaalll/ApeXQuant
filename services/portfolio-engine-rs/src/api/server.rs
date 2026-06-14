use axum::Router;
use std::net::SocketAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tonic::transport::Server as TonicServer;
use tracing::{info, error};

use apex_protos::portfolio::portfolio_engine_server::PortfolioEngineServer;
use crate::api::portfolio_service::PortfolioServiceImpl;
use crate::health::api::{health_routes, HealthState};
use crate::interceptors::logging::logging_interceptor;

pub async fn start_server(addr: SocketAddr) -> anyhow::Result<()> {
    let health_state = Arc::new(HealthState {
        active_tasks: AtomicUsize::new(0),
    });

    let http_router = health_routes(health_state.clone());

    let grpc_router = TonicServer::builder()
        .add_service(PortfolioEngineServer::with_interceptor(
            PortfolioServiceImpl::new(),
            logging_interceptor,
        ))
        .into_router();

    // Merge HTTP routes and gRPC routes into one Axum router
    let app = Router::new()
        .merge(http_router)
        .merge(grpc_router);

    info!("Starting Portfolio Engine server on {}", addr);
    
    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        error!("Server error: {}", e);
    }

    Ok(())
}
