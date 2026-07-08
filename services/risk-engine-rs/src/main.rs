#![deny(unsafe_code)]

use sqlx::postgres::PgPoolOptions;
use redis::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Initializing Risk Engine V1 Phase 13.4: Production Completion...");

    let config = risk_engine::config::RiskConfig::from_env().unwrap_or_else(|e| {
        tracing::error!("Configuration error: {}", e);
        std::process::exit(1);
    });

    // Initialize Postgres Pool using real config
    let pg_pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;
        
    // Initialize Redis using real config
    let redis_client = Client::open(config.redis_url.clone())?;

    // Initialize EventBus
    let event_bus = match risk_engine::event_bus::EventBusPublisher::connect(config.eventbus_url.clone()).await {
        Ok(publisher) => {
            tracing::info!("Event Bus connected at {}", config.eventbus_url);
            Some(std::sync::Arc::new(publisher))
        }
        Err(e) => {
            tracing::error!("Failed to connect to EventBus at {}: {}", config.eventbus_url, e);
            std::process::exit(1);
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
        
        if let Ok(mut rx) = subscriber.subscribe("market.tick.*").await {
            let _state_clone = risk_state.clone();
            tokio::spawn(async move {
                while let Some(_event) = rx.recv().await {
                    // Update risk states based on market ticks
                }
            });
        }
    }

    // Initialize the storage layer
    let _risk_repository = risk_engine::storage::repository::RiskRepository::new(pg_pool.clone());

    // Setup graceful shutdown signal
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to listen for event");
        tracing::info!("Received shutdown signal");
        let _ = shutdown_tx.send(()).await;
    });

    // Start multiplexed gRPC server
    tokio::select! {
        res = risk_engine::api::server::start_server(risk_state, event_bus, pg_pool.clone(), redis_client.clone()) => {
            if let Err(e) = res {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = shutdown_rx.recv() => {
            tracing::info!("Shutting down risk engine server gracefully.");
        }
    }

    Ok(())
}

