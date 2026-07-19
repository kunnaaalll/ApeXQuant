#![allow(warnings, clippy::all, deprecated)]
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tracing::{error, info, Level};

use portfolio_engine::event_bus::EventBusPublisher;
use portfolio_engine::event_bus_subscriber::EventBusSubscriber;
use portfolio_engine::exposure::registry::ExposureRegistry;
use portfolio_engine::portfolio::registry::PortfolioRegistry;
use portfolio_engine::storage::pg_store::PostgresPortfolioStore;
use portfolio_engine::storage::repository::PortfolioRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

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
    let exposure_registry = ExposureRegistry::new();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    info!("Connecting to Redis...");
    let redis_client = match redis::Client::open(redis_url) {
        Ok(client) => Some(client),
        Err(e) => {
            error!("Redis connection failed (non-fatal): {:?}", e);
            None
        }
    };

    info!("Connecting to Event Bus...");
    let event_bus_url =
        env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    let event_bus = match EventBusPublisher::connect(event_bus_url.clone()).await {
        Ok(bus) => {
            info!("Event Bus connected");
            Some(Arc::new(bus))
        }
        Err(e) => {
            error!("Event Bus connection failed (non-fatal): {:?}", e);
            None
        }
    };

    // Initialize Event Bus Subscriber
    info!("Initializing Event Bus Subscriber...");
    match EventBusSubscriber::connect(
        event_bus_url,
        "portfolio_group".to_string(),
        "portfolio_instance_1".to_string(),
    )
    .await
    {
        Ok(subscriber) => {
            info!("Subscriber connected to Event Bus. Starting stream subscriptions...");

            if let Ok(rx_opened) = subscriber.subscribe("execution.position.opened").await {
                subscriber
                    .start_listening(
                        rx_opened,
                        registry.clone(),
                        exposure_registry.clone(),
                        pool.clone(),
                        event_bus.clone(),
                    )
                    .await;
            }
            if let Ok(rx_closed) = subscriber.subscribe("execution.position.closed").await {
                subscriber
                    .start_listening(
                        rx_closed,
                        registry.clone(),
                        exposure_registry.clone(),
                        pool.clone(),
                        event_bus.clone(),
                    )
                    .await;
            }
            if let Ok(rx_ticks) = subscriber.subscribe("market.tick.*").await {
                subscriber
                    .start_listening(
                        rx_ticks,
                        registry.clone(),
                        exposure_registry.clone(),
                        pool.clone(),
                        event_bus.clone(),
                    )
                    .await;
            }
        }
        Err(e) => {
            error!("Failed to initialize Event Bus Subscriber: {:?}", e);
        }
    }

    portfolio_engine::rebalancing::RebalanceEngine::spawn_reconciliation_loop(
        30,
        exposure_registry.clone(),
        pool.clone(),
        event_bus.clone(),
    );

    let addr = "0.0.0.0:50051".parse()?;
    portfolio_engine::api::server::start_server(
        addr,
        event_bus,
        pool,
        redis_client,
        registry,
        exposure_registry,
        repository,
    )
    .await?;

    info!("Portfolio Engine is shutting down.");
    Ok(())
}
