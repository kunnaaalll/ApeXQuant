#![deny(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Portfolio Engine...");
    
    let addr = "0.0.0.0:50051".parse()?;
    portfolio_engine::api::server::start_server(addr).await?;

    info!("Portfolio Engine is shutting down.");
    Ok(())
}
