use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use position_engine::positions::{PositionRegistry, PositionManager};
use position_engine::storage::PostgresStore;
use position_engine::api::PositionEngineService;
use apex_protos::position::position_engine_server::PositionEngineServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting APEX V3 Position Engine...");

    // 2. Load configurations
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/apex_event_bus".to_string());
    let mt5_bridge_url = std::env::var("MT5_BRIDGE_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let grpc_port = std::env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50054".to_string());

    // 3. Connect to Database
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let store = Arc::new(PostgresStore::new(pool));
    let registry = PositionRegistry::new();

    // 4. Start Position Manager Background Thread
    info!("Starting Position Manager sync loop pointing to {}", mt5_bridge_url);
    let manager = Arc::new(PositionManager::new(registry.clone(), store.clone(), mt5_bridge_url.clone()));
    manager.start();

    // 5. Start gRPC Service Server
    let addr: SocketAddr = format!("[::]:{}", grpc_port).parse()?;
    info!("Starting Position Engine gRPC API Server on {}", addr);

    let service = PositionEngineService::new(registry, store, mt5_bridge_url);

    Server::builder()
        .add_service(PositionEngineServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
