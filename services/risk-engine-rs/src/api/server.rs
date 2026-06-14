use std::sync::Arc;
use tonic::transport::Server;
use tracing::{info, instrument};

use apex_protos::risk::risk_engine_server::RiskEngineServer;

use crate::{config::RiskEngineConfig, RiskEngine};
use super::service::RiskService;
use super::interceptors::metrics_interceptor;

use std::future::Future;

#[instrument(skip(engine, config, shutdown_signal))]
pub async fn start_grpc_server<F>(
    engine: Arc<RiskEngine>,
    config: &RiskEngineConfig,
    shutdown_signal: F,
) -> Result<(), anyhow::Error>
where
    F: Future<Output = ()> + Send + 'static,
{
    let addr = format!("{}:{}", config.grpc_host, config.grpc_port).parse()?;
    info!("Starting Risk Engine gRPC server on {}", addr);

    let service = RiskService::new(engine);
    let server = RiskEngineServer::with_interceptor(service, metrics_interceptor);

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<RiskEngineServer<RiskService>>().await;

    Server::builder()
        .add_service(health_service)
        .add_service(server)
        .serve_with_shutdown(addr, shutdown_signal)
        .await?;

    Ok(())
}
