#![allow(warnings, clippy::all, deprecated)]
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tracing::{info, warn};

use strategy_engine_rs::api::server::start_server;
use strategy_engine_rs::api::service::StrategyState;
use strategy_engine_rs::event_bus_subscriber::EventBusSubscriber;
use strategy_engine_rs::meta::strategy_registry::StrategyRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting APEX V3 Strategy Engine...");

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/apex_strategy".to_string()
    });
    let _redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let event_bus_url =
        env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50051".to_string());

    let grpc_addr: SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50053".to_string())
        .parse()?;

    let http_addr: SocketAddr = env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8083".to_string())
        .parse()?;

    info!("Connecting to PostgreSQL at {}", db_url);
    let _pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    info!("Connecting to EventBus at {}", event_bus_url);
    let subscriber = EventBusSubscriber::connect(
        event_bus_url.clone(),
        "strategy-engine-group".to_string(),
        "strategy-engine-instance-1".to_string(),
    )
    .await;

    if let Ok(sub) = subscriber {
        info!("Subscribing to Market Data, Signal Engine, and Risk Engine events...");

        let mut signal_rx = sub.subscribe("signals.detected").await?;
        tokio::spawn(async move {
            while let Some(event) = signal_rx.recv().await {
                info!("Received signal event: {:?}", event.event_type);
            }
        });
    } else {
        warn!("Failed to connect to EventBus. Proceeding without event subscriptions for now.");
    }

    let _registry = StrategyRegistry::new();
    let state = StrategyState::new();

    info!(
        "Starting gRPC server on {} and HTTP health server on {}",
        grpc_addr, http_addr
    );

    // Start server, and await shutdown
    tokio::select! {
        res = start_server(grpc_addr, http_addr, state) => {
            if let Err(e) = res {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl-C, initiating graceful shutdown...");
        }
    }

    info!("Shutdown complete.");
    Ok(())
}
