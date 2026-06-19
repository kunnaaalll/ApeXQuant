#![deny(unsafe_code)]

use sqlx::postgres::PgPoolOptions;
use redis::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Initializing Risk Engine V1 Phase 10: API Layer...");

    // Initialize Postgres Pool (placeholder connection string for now)
    let _pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy("postgres://postgres:postgres@localhost:5432/apex")?;
        
    // Initialize Redis (placeholder connection string)
    let _redis_client = Client::open("redis://127.0.0.1:6379/")?;

    // Start multiplexed server
    risk_engine::api::server::start_server().await?;

    Ok(())
}
