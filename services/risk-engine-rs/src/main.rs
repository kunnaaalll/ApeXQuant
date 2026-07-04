#![deny(unsafe_code)]

use sqlx::postgres::PgPoolOptions;
use redis::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Initializing Risk Engine V1 Phase 10: API Layer...");

    // Initialize Postgres Pool (placeholder connection string for now)
    let _pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy("postgres://postgres:postgres@localhost:5432/apex")?;
        
    // Initialize Redis (placeholder connection string)
    let _redis_client = Client::open("redis://127.0.0.1:6379/")?;

    // Initialize EventBus
    let config = risk_engine::config::RiskConfig::from_env().unwrap();
    let event_bus = match risk_engine::event_bus::EventBusPublisher::connect(config.eventbus_url.clone()).await {
        Ok(publisher) => {
            tracing::info!("Event Bus connected at {}", config.eventbus_url);
            Some(std::sync::Arc::new(publisher))
        }
        Err(e) => {
            tracing::warn!("Failed to connect to EventBus at {}: {}", config.eventbus_url, e);
            None
        }
    };

    let risk_state = risk_engine::api::risk_service::RiskState::new();

    // Integrate position data stream for correlation and exposure
    if let Ok(subscriber) = risk_engine::event_bus_subscriber::EventBusSubscriber::connect(
        config.eventbus_url.clone(),
        "risk_engine_group".to_string(),
        uuid::Uuid::new_v4().to_string(),
    ).await {
        if let Ok(mut rx) = subscriber.subscribe("execution.position").await {
            let state_clone = risk_state.clone();
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    if let Some(payload) = event.payload {
                        use apex_protos::events::event::Payload;
                        match payload {
                            Payload::PositionOpened(pos) => {
                                let mut corr = state_clone.correlation.write().await;
                                corr.set_correlation("Symbol", &pos.symbol, "EURUSD", rust_decimal_macros::dec!(0.5));
                                corr.set_correlation("Symbol", &pos.symbol, "GBPUSD", rust_decimal_macros::dec!(0.4));
                                
                                let mut exp = state_clone.exposure.write().await;
                                let vol = pos.initial_volume.and_then(|v| v.units.parse::<rust_decimal::Decimal>().ok()).unwrap_or_default();
                                exp.gross_exposure += vol;
                                if pos.side == 1 { exp.net_exposure += vol; } else { exp.net_exposure -= vol; }
                            }
                            Payload::PositionClosed(pos) => {
                                let mut exp = state_clone.exposure.write().await;
                                let vol = pos.closed_volume.and_then(|v| v.units.parse::<rust_decimal::Decimal>().ok()).unwrap_or_default();
                                exp.gross_exposure -= vol;
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
    }

    // Start multiplexed gRPC server
    risk_engine::api::server::start_server(
        risk_state,
        event_bus,
    ).await?;

    Ok(())
}
