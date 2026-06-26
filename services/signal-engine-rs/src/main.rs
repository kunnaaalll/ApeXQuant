//! APEX V3 Signal Engine - Main Entry Point
//!
//! This is the service entry point. For the library API, see lib.rs.

use signal_engine::{Config, Result, SignalEngine};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("APEX V3 Signal Engine starting...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Initialize EventBus
    let event_bus_url = std::env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    let event_bus = match signal_engine::event_bus::EventBusPublisher::connect(event_bus_url.clone()).await {
        Ok(publisher) => {
            info!("Successfully connected to EventBus at {}", event_bus_url);
            Some(Arc::new(publisher))
        }
        Err(e) => {
            warn!("Failed to connect to EventBus at {}: {}", event_bus_url, e);
            None
        }
    };

    // Initialize signal engine
    let engine = SignalEngine::new(config, event_bus).await?;
    info!("Signal engine initialized");

    // Run server (placeholder - would integrate with gRPC/HTTP server)
    info!("Starting signal engine server...");

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping...");
        }
        Err(e) => {
            warn!("Failed to listen for shutdown signal: {}", e);
        }
    }

    info!("APEX Signal Engine stopped");
    Ok(())
}
