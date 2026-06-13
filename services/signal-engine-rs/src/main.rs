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
    let config = Config::from_env()?;
    info!("Configuration loaded");

    // Initialize signal engine
    let engine = SignalEngine::new(config).await?;
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
