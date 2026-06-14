use anyhow::Result;
use risk_engine::api::start_grpc_server;
use risk_engine::config::RiskEngineConfig;
use risk_engine::RiskEngine;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize tracing and OpenTelemetry
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    // 2. Set up panic hook for proper logging
    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };
        let location = info.location().unwrap();
        tracing::error!(
            panic_location = %location,
            panic_message = %msg,
            "Application panicked!"
        );
    }));

    info!("Starting Risk Engine Service...");

    // 3. Load configuration
    let config = RiskEngineConfig::load()?;
    
    // 4. Initialize core components
    let risk_engine = Arc::new(RiskEngine::new(config.clone(), None));
    
    // 5. Initialize health and metrics server
    let metrics_handle = risk_engine::health::setup_metrics();
    let health_port = config.health_port;
    tokio::spawn(async move {
        risk_engine::health::start_health_server(health_port, metrics_handle).await;
    });

    // 6. Initialize gRPC server
    start_grpc_server(risk_engine, &config, shutdown_signal()).await?;

    info!("Risk Engine Service shut down cleanly");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl-C signal, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, starting graceful shutdown");
        },
    }
}
