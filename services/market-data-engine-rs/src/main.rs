use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting APEX V3 Market Data Engine...");

    // Placeholder: keep alive until full engine logic is wired in
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
