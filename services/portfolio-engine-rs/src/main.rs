#![deny(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use tracing::{info, error, Level};
use std::sync::Arc;
use portfolio_engine::event_bus::EventBusPublisher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Portfolio Engine...");
    info!("Connecting to Event Bus...");
    // Retrieve EVENT_BUS_URL from environment or default
    let event_bus_url = std::env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
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
    portfolio_engine::api::server::start_server(addr, event_bus).await?;

    info!("Portfolio Engine is shutting down.");
    Ok(())
}
