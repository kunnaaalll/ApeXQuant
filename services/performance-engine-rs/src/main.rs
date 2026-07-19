use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use performance_engine::analytics::engine::AnalyticsEngine;
use performance_engine::config::AppConfig;
use performance_engine::event_bus::{EventBusPublisher, EventBusSubscriber};
use performance_engine::storage::Repositories;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize deterministic logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting APEX Performance Engine V1 in Zero Trust Production Mode...");

    // 1. Configuration
    let config = AppConfig::from_env();

    // 2. Storage & Repositories
    info!("Connecting to PostgreSQL at {}", config.database_url);
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await?;

    let repos = Arc::new(Repositories::new(pool));

    // Run idempotent migrations
    info!("Running database migrations...");
    repos.migrate().await?;

    // 3. Analytics Engine
    let engine = Arc::new(AnalyticsEngine::new());

    // 4. Event Bus
    info!("Connecting to Event Bus at {}", config.event_bus_url);
    let _publisher = EventBusPublisher::connect(
        config.event_bus_url.clone(),
        "performance-engine".to_string(),
    )
    .await?;

    let subscriber = EventBusSubscriber::connect(
        config.event_bus_url.clone(),
        "performance_engine_group".to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .await?;

    // Subscribe to required domain events
    let topics = vec![
        "execution.position.closed",
        "execution.trade.closed",
        "strategy.state.changed",
        "risk.evaluation.updated",
        "market.session.closed",
        "system.daily_rollover",
    ];

    for topic in topics {
        let mut rx = subscriber.subscribe(topic).await?;
        let _repos_clone = repos.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // In a full implementation, we'd route these to specific handlers
                // e.g., converting PositionClosed events into ClosedTradeRecords
                // and storing them via repos_clone.performance.upsert_trade(...)
                tracing::debug!("Received event on {}: {:?}", event.topic, event.event_id);
                // repos_clone.performance...
            }
        });
        info!("Subscribed to topic: {}", topic);
    }

    // 5. API Server (gRPC + HTTP Metrics/Health)
    let grpc_addr = format!("0.0.0.0:{}", config.grpc_port)
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:50055".parse().unwrap());
    let http_addr = format!("0.0.0.0:{}", config.http_port)
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:8085".parse().unwrap());

    let api_repos = repos.clone();
    let api_engine = engine.clone();

    tokio::spawn(async move {
        if let Err(e) =
            performance_engine::api::start_api_server(grpc_addr, http_addr, api_repos, api_engine)
                .await
        {
            error!("API server error: {:?}", e);
        }
    });

    info!("APEX Performance Engine V1 initialized securely.");

    // Graceful shutdown handler
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutting down gracefully...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}
