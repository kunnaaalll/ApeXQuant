pub mod service;

use std::net::SocketAddr;
use apex_protos::analytics::analytics_engine_server::AnalyticsEngineServer;
use tonic::transport::Server;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn start_api_server(
    addr: SocketAddr,
    state: Arc<RwLock<service::PerformanceState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting Performance Engine gRPC server on {}", addr);
    
    let analytics_service = service::AnalyticsServiceImpl {
        state,
    };

    Server::builder()
        .add_service(AnalyticsEngineServer::new(analytics_service))
        .serve(addr)
        .await?;

    Ok(())
}
