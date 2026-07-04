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
    let engine = Arc::new(SignalEngine::new(config, event_bus).await?);
    info!("Signal engine initialized");

    // Run server
    let addr = "0.0.0.0:50051".parse().map_err(|e: std::net::AddrParseError| signal_engine::SignalEngineError::Validation(e.to_string()))?;
    let server_fut = signal_engine::api::server::start_server(engine.clone(), addr);


    // Wait for shutdown signal or server exit
    tokio::select! {
        res = server_fut => {
            if let Err(e) = res {
                warn!("gRPC server exited with error: {:?}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received, stopping...");
        }
    }


    info!("APEX Signal Engine stopped");
    Ok(())
}
