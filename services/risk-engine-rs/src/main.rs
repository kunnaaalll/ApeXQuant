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

    // Initialize EventBus
    let event_bus_url = std::env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    let event_bus = match risk_engine::event_bus::EventBusPublisher::connect(event_bus_url.clone()).await {
        Ok(publisher) => {
            tracing::info!("Successfully connected to EventBus at {}", event_bus_url);
            Some(std::sync::Arc::new(publisher))
        }
        Err(e) => {
            tracing::warn!("Failed to connect to EventBus at {}: {}", event_bus_url, e);
            None
        }
    };

    // Start multiplexed server
    risk_engine::api::server::start_server(risk_engine::api::risk_service::RiskState::new(), event_bus).await?;

    Ok(())
}
