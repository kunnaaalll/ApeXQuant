#![deny(unsafe_code)]

use execution_engine::api::server::start_api_servers;
use execution_engine::broker_supervisor::BrokerSupervisor;
use execution_engine::brokers::binance::adapter::BinanceAdapter;
use execution_engine::brokers::mt5::adapter::Mt5Adapter;
use execution_engine::config::EnvironmentConfiguration;
use execution_engine::event_bus::EventBusPublisher;
use execution_engine::api::readiness;

use std::sync::Arc;
use tracing::{error, info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Step 1: Load configuration
    let config = EnvironmentConfiguration::from_env();

    // Step 2: Initialize observability
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting APEX V3 Execution Engine...");

    // Step 3: Initialize broker adapters
    info!("Initializing Mt5Adapter...");
    let mt5_adapter = Arc::new(Mt5Adapter::new(
        "MT5_PROD_1".to_string(),
        config.mt5_bridge_url.clone(),
    ));
    if let Err(e) = mt5_adapter.connect().await {
        error!("Mt5Adapter initial connection failed: {:?}", e);
    }

    info!("Initializing BinanceAdapter...");
    let binance_adapter = Arc::new(BinanceAdapter::new(
        "BINANCE_FUTURES_1".to_string(),
        config.binance_base_url.clone(),
        config.binance_api_key.clone(),
        config.binance_secret.clone(),
    ));
    if let Err(e) = binance_adapter.connect().await {
        error!("BinanceAdapter initial connection failed: {:?}", e);
    }

    // Step 4: Connect event bus
    info!("Connecting to Event Bus...");
    let event_bus = match EventBusPublisher::connect(config.event_bus_url.clone()).await {
        Ok(bus) => {
            info!("Event Bus connected");
            Some(Arc::new(bus))
        }
        Err(e) => {
            error!("Event Bus connection failed (non-fatal): {:?}", e);
            None
        }
    };

    // Step 5: Start reconciliation and background loops
    let supervisor = BrokerSupervisor::new(Arc::clone(&mt5_adapter), Arc::clone(&binance_adapter));
    supervisor.spawn_heartbeat_loop(config.heartbeat_interval_secs);
    supervisor.spawn_health_loop(config.health_check_interval_secs);
    supervisor.spawn_reconciliation_loop(config.reconciliation_interval_secs);

    // Step 6: Start gRPC server
    info!(
        "Starting API servers (gRPC: {}, HTTP: {})...",
        config.grpc_port, config.http_port
    );
    let (grpc_handle, http_handle) = start_api_servers(config.grpc_port, config.http_port, event_bus.clone()).await;

    // Step 7: Publish readiness
    readiness::set_ready(true);
    info!("Execution Engine is READY.");

    // Step 8: Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received.");
        }
        res = grpc_handle => {
            error!("gRPC server exited: {:?}", res);
        }
        res = http_handle => {
            error!("HTTP server exited: {:?}", res);
        }
    }

    readiness::set_ready(false);
    info!("Shutting down broker connections...");
    let _ = mt5_adapter.shutdown().await;
    let _ = binance_adapter.shutdown().await;

    info!("Execution Engine shutdown complete.");
    Ok(())
}
