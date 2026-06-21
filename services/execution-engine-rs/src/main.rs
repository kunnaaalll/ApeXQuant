#![deny(unsafe_code)]

use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Execution Engine...");

    info!("Execution Engine shutdown.");
    Ok(())
}
