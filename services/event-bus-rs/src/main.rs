#![cfg_attr(not(test), deny(unsafe_code))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

use anyhow::{Result, Context};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tonic::transport::Server;
use apex_protos::events::event_bus_service_server::EventBusServiceServer;

use event_bus::storage::EventStore;
use event_bus::routing::SequenceManager;
use event_bus::server::EventBusServiceImpl;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/apex_event_bus".to_string());
    
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    let store = EventStore::new(pool.clone());
    
    // Initialize schema
    tracing::info!("Initializing database schema...");
    store.init().await?;

    let sequencer = SequenceManager::new(pool.clone());
    let service_impl = EventBusServiceImpl::new(store, sequencer);
    
    let addr = "[::]:50051".parse()?;
    
    tracing::info!("APEX V3 Event Bus starting on {}", addr);

    Server::builder()
        .add_service(EventBusServiceServer::new(service_impl))
        .serve(addr)
        .await?;

    Ok(())
}
