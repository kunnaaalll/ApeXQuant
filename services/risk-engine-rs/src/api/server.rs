use std::net::SocketAddr;
use tower::ServiceBuilder;

use apex_protos::risk::risk_engine_server::RiskEngineServer;

use crate::api::risk_service::RiskServiceImpl;
use crate::health::api::health_routes;
use crate::interceptors::auth::auth_interceptor;
use crate::interceptors::logging::LoggingLayer;
use crate::interceptors::metrics::MetricsLayer;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "0.0.0.0:50053".parse()?;

    let health_router = health_routes();

    let grpc_service = RiskEngineServer::with_interceptor(RiskServiceImpl, auth_interceptor);

    let grpc_router = tonic::transport::Server::builder()
        .layer(ServiceBuilder::new()
            .layer(LoggingLayer)
            .layer(MetricsLayer)
        )
        .add_service(grpc_service)
        .into_router();

    let app = health_router.merge(grpc_router);

    tracing::info!("Starting server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
