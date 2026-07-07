use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Strategy Engine...");
    
    // The engine is still under development, keep it alive so docker-compose doesn't crash
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
