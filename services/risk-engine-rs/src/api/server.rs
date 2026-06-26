use std::net::SocketAddr;
use tower::ServiceBuilder;

use apex_protos::risk::risk_engine_server::RiskEngineServer;

use crate::api::risk_service::{RiskServiceImpl, RiskState};
use crate::health::api::health_routes;
use crate::interceptors::auth::auth_interceptor;
use crate::interceptors::logging::LoggingLayer;
use crate::interceptors::metrics::MetricsLayer;

use std::sync::Arc;
use crate::event_bus::EventBusPublisher;

/// Start the risk engine gRPC + HTTP health server.
///
/// `state` is the live `RiskState` initialised at service bootstrap.
/// All gRPC handlers read from this shared state — no data constructed inside handlers.
pub async fn start_server(
    state: RiskState,
    event_bus: Option<Arc<EventBusPublisher>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "0.0.0.0:50053".parse()?;

    let health_router = health_routes();

    let grpc_service = RiskEngineServer::with_interceptor(
        RiskServiceImpl::new(state, event_bus),
        auth_interceptor,
    );

    let grpc_router = tonic::transport::Server::builder()
        .layer(
            ServiceBuilder::new()
                .layer(LoggingLayer)
                .layer(MetricsLayer),
        )
        .add_service(grpc_service)
        .into_router();

    let app = health_router.merge(grpc_router);

    tracing::info!(%addr, "RiskEngine: starting server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
