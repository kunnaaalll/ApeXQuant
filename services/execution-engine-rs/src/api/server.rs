use std::net::SocketAddr;
use tonic::transport::Server;
use apex_protos::execution::execution_engine_server::ExecutionEngineServer;
use super::service::ExecutionService;

pub async fn start_grpc_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let service = ExecutionService::new();
    
    println!("Execution Engine gRPC server listening on {}", addr);
    
    Server::builder()
        .add_service(ExecutionEngineServer::new(service))
        .serve(addr)
        .await?;
        
    Ok(())
}
