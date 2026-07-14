#![cfg_attr(not(test), deny(unsafe_code))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

use anyhow::{Context, Result};
use apex_protos::events::event_bus_service_server::EventBusServiceServer;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::str::FromStr;
use tonic::transport::Server;

use event_bus::config::Config;
use event_bus::metrics::init_telemetry;
use event_bus::nats::NatsManager;
use event_bus::redis::RedisManager;
use event_bus::router::SequenceManager;
use event_bus::server::EventBusServiceImpl;
use event_bus::storage::EventStore;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Validate Configuration
    let config = Config::from_env().context("Failed to load configuration")?;

    // 2. Initialize Telemetry & Tracing
    init_telemetry("event-bus").context("Failed to initialize telemetry")?;
    tracing::info!("Starting APEX V3 Event Bus");

    // 3. Initialize PostgreSQL
    tracing::info!("Connecting to PostgreSQL at {}", config.database_url);
    let pg_pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    let store = EventStore::new(pg_pool.clone());
    tracing::info!("Initializing database schema...");
    store.init().await?;

    // 4. Initialize Redis
    tracing::info!("Connecting to Redis...");
    let redis_manager = RedisManager::connect(&config.redis_url).await?;

    // 5. Initialize NATS
    tracing::info!("Connecting to NATS JetStream...");
    let _nats_manager = NatsManager::connect(&config.nats_url).await?;

    // 6. Initialize Routing & Sequencer
    let sequencer = SequenceManager::new(redis_manager.clone());

    // 7. Initialize gRPC Service
    let service_impl = EventBusServiceImpl::new(store.clone(), sequencer);

    let addr = SocketAddr::from_str(&config.bind_address).context("Invalid bind address")?;

    tracing::info!("APEX V3 Event Bus starting on {}", addr);

    // 8. Graceful Shutdown & Expose gRPC
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutting down Event Bus...");
        let _ = tx.send(());
    });

    Server::builder()
        .add_service(EventBusServiceServer::new(service_impl))
        .serve_with_shutdown(addr, async {
            rx.await.ok();
        })
        .await?;

    event_bus::metrics::shutdown_telemetry();
    Ok(())
}
