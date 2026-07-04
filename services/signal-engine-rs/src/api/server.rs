use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use apex_protos::signal::signal_engine_server::SignalEngineServer;
use crate::api::service::SignalEngineServiceImpl;
use crate::SignalEngine;

pub async fn start_server(
    engine: Arc<SignalEngine>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = SignalEngineServiceImpl::new(engine);

    tracing::info!("Starting Signal Engine gRPC server on {}", addr);

    Server::builder()
        .add_service(SignalEngineServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
