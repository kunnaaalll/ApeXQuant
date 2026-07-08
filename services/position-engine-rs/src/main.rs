use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use async_nats;

use position_engine::positions::{PositionRegistry, PositionManager};
use position_engine::storage::PostgresStore;
use position_engine::api::PositionEngineService;
use position_engine::event_bus::{EventPublisher, EventSubscriber};
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
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/apex_position".to_string());
    let mt5_bridge_url = std::env::var("MT5_BRIDGE_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let grpc_port = std::env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50054".to_string());
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    // 3. Connect to Database and run migrations
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    info!("Running database migrations...");
    // Assuming migrations are manually run for this environment or setup externally via CI.
    // If not, we can use sqlx::migrate!("./migrations").run(&pool).await?;

    let store = Arc::new(PostgresStore::new(pool));
    let registry = PositionRegistry::new();

    // 4. Connect to NATS Event Bus
    info!("Connecting to NATS event bus at {}", nats_url);
    let nats_client = async_nats::connect(&nats_url).await?;
    let _publisher = EventPublisher::new(nats_client.clone());
    // EventSubscriber takes Arc<PositionRegistry> for shared ownership across tasks
    let subscriber = EventSubscriber::new(nats_client, Arc::new(registry.clone()));
    subscriber.start().await?;

    // 5. Start Position Manager Background Thread
    info!("Starting Position Manager sync loop pointing to {}", mt5_bridge_url);
    let manager = Arc::new(PositionManager::new(registry.clone(), store.clone(), mt5_bridge_url.clone()));
    manager.start();

    // 6. Start gRPC Service Server
    let addr: SocketAddr = format!("[::]:{}", grpc_port).parse()?;
    info!("Starting Position Engine gRPC API Server on {}", addr);

    let service = PositionEngineService::new(registry, store, mt5_bridge_url);

    Server::builder()
        .add_service(PositionEngineServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
