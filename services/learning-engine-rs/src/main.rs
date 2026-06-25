#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use learning_engine::bus::EventBusIntegration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("APEX V3 Learning Engine - Starting up...");
    
    // Initialize event bus integration
    let _bus = EventBusIntegration::new();
    
    // Keep engine running
    // std::future::pending::<()>().await;
    
    println!("Learning Engine shutting down.");
    Ok(())
}
