use tracing::{info, error, Level};
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use std::env;

use portfolio_engine::event_bus::EventBusPublisher;
use portfolio_engine::portfolio::registry::PortfolioRegistry;
use portfolio_engine::storage::repository::PortfolioRepository;
use portfolio_engine::storage::pg_store::PostgresPortfolioStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Portfolio Engine...");

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://apex:apex_password@localhost:5432/apex_v3".to_string());

    info!("Connecting to PostgreSQL database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let store = PostgresPortfolioStore::new(pool.clone());
    info!("Initializing Portfolio database schema...");
    store.init_tables().await?;

    let repository = PortfolioRepository::new(store);
    let registry = PortfolioRegistry::new();

    info!("Connecting to Event Bus...");
    let event_bus_url = env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    let event_bus = match EventBusPublisher::connect(event_bus_url).await {
        Ok(bus) => {
            info!("Event Bus connected");
            Some(Arc::new(bus))
        }
        Err(e) => {
            error!("Event Bus connection failed (non-fatal): {:?}", e);
            None
        }
    };
    
    portfolio_engine::rebalancing::RebalanceEngine::spawn_reconciliation_loop(30);

    let addr = "0.0.0.0:50051".parse()?;
    portfolio_engine::api::server::start_server(addr, event_bus, pool, registry, repository).await?;

    info!("Portfolio Engine is shutting down.");
    Ok(())
}
