use axum::Router;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tonic::transport::Server as TonicServer;
use tracing::{error, info};

use crate::api::portfolio_service::PortfolioServiceImpl;
use crate::event_bus::EventBusPublisher;
use crate::health::api::{health_routes, HealthState};
use crate::interceptors::logging::logging_interceptor;
use crate::portfolio::registry::PortfolioRegistry;
use crate::storage::repository::PortfolioRepository;

pub async fn start_server(
    addr: SocketAddr,
    event_bus: Option<Arc<EventBusPublisher>>,
    pool: PgPool,
    redis_client: Option<redis::Client>,
    registry: PortfolioRegistry,
    exposure_registry: crate::exposure::registry::ExposureRegistry,
    repository: PortfolioRepository,
) -> anyhow::Result<()> {
    let health_state = Arc::new(HealthState {
        active_tasks: AtomicUsize::new(0),
        pool: pool.clone(),
        redis_client,
    });

    let http_router = health_routes(health_state.clone());

    let grpc_router = TonicServer::builder()
        .add_service(apex_protos::portfolio::portfolio_engine_server::PortfolioEngineServer::with_interceptor(
            PortfolioServiceImpl::new(event_bus, pool, registry, exposure_registry, repository),
            logging_interceptor,
        ))
        .into_router();

    // Merge HTTP routes and gRPC routes into one Axum router
    let app = Router::new().merge(http_router).merge(grpc_router);

    info!("Starting Portfolio Engine server on {}", addr);

    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        error!("Server error: {}", e);
    }

    Ok(())
}
