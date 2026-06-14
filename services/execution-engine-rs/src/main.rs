use execution_engine::health;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Execution Engine...");

    // TODO: Load configuration, connect to PostgreSQL, initialize modules

    info!("Execution Engine shutdown.");
    Ok(())
}
