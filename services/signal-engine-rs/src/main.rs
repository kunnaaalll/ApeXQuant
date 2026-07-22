#![allow(warnings, clippy::all, deprecated)]
//! APEX V3 Signal Engine - Main Entry Point
//!
//! This is the service entry point. For the library API, see lib.rs.

use signal_engine::{Config, Result, SignalEngine};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("APEX V3 Signal Engine starting...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Initialize EventBus
    let event_bus_url =
        std::env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    let event_bus =
        match signal_engine::event_bus::EventBusPublisher::connect(event_bus_url.clone()).await {
            Ok(publisher) => {
                info!("Successfully connected to EventBus at {}", event_bus_url);
                Some(Arc::new(publisher))
            }
            Err(e) => {
                warn!("Failed to connect to EventBus at {}: {}", event_bus_url, e);
                None
            }
        };

    // Initialize Database
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:signals.db".to_string());
    let pool = sqlx::SqlitePool::connect(&db_url)
        .await
        .unwrap_or_else(|_| {
            // Fallback to in-memory for dev if file fails
            sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap_or_else(|e| {
                tracing::error!("Fallback DB failed: {}", e);
                std::process::exit(1)
            })
        });

    let repository = Arc::new(signal_engine::storage::SignalRepository::new(pool));
    if let Err(e) = repository.initialize().await {
        warn!("Failed to initialize database schema: {}", e);
    }

    // Initialize signal engine
    let engine = Arc::new(SignalEngine::new(config, event_bus, Some(repository)).await?);
    info!("Signal engine initialized");

    // Run server
    let addr = "0.0.0.0:50051"
        .parse()
        .map_err(|e: std::net::AddrParseError| {
            signal_engine::SignalEngineError::Validation(e.to_string())
        })?;
    let server_fut = signal_engine::api::server::start_server(engine.clone(), addr);

    // Initialize EventBus subscriber
    let engine_clone = engine.clone();
    let event_bus_url_sub = event_bus_url.clone();
    let subscriber_fut = tokio::spawn(async move {
        if let Ok(subscriber) =
            signal_engine::event_bus::EventBusSubscriber::connect(event_bus_url_sub).await
        {
            info!("Successfully connected EventBusSubscriber");
            let topics = vec![
                "market_data.candle_closed".to_string(),
                "market_data.tick_received".to_string(),
            ];
            if let Ok(mut stream) = subscriber
                .subscribe(
                    "signal_engine".to_string(),
                    "signal_engine_1".to_string(),
                    topics,
                )
                .await
            {
                while let Ok(Some(batch)) = stream.message().await {
                    for event in batch.events {
                        if let Some(payload) = event.payload {
                            match payload {
                                apex_protos::events::event::Payload::CandleClosed(candle_event) => {
                                    // Parse candle and pass to engine
                                    use std::str::FromStr;
                                    let open = rust_decimal::Decimal::from_str(
                                        &candle_event.open.unwrap_or_default().value,
                                    )
                                    .unwrap_or_default();
                                    let high = rust_decimal::Decimal::from_str(
                                        &candle_event.high.unwrap_or_default().value,
                                    )
                                    .unwrap_or_default();
                                    let low = rust_decimal::Decimal::from_str(
                                        &candle_event.low.unwrap_or_default().value,
                                    )
                                    .unwrap_or_default();
                                    let close = rust_decimal::Decimal::from_str(
                                        &candle_event.close.unwrap_or_default().value,
                                    )
                                    .unwrap_or_default();

                                    let timestamp = if let Some(ct) = candle_event.close_time {
                                        time::OffsetDateTime::from_unix_timestamp(ct.seconds)
                                            .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                                    } else {
                                        time::OffsetDateTime::now_utc()
                                    };

                                    let volume_u64 = candle_event
                                        .volume
                                        .map(|v| v.units.parse::<f64>().unwrap_or(0.0) as u64)
                                        .unwrap_or(0);
                                    let candle = signal_engine::market_data::Candle::new(
                                        timestamp, open, high, low, close, volume_u64,
                                    );

                                    let timeframe_str = if let Some(tf) = candle_event.timeframe {
                                        let unit_str = match tf.unit {
                                            1 => "M",
                                            2 => "H",
                                            3 => "D",
                                            _ => "M",
                                        };
                                        format!("{}{}", unit_str, tf.value)
                                    } else {
                                        "M15".to_string() // Fallback
                                    };

                                    if let Err(e) = engine_clone
                                        .process_market_data(
                                            &candle_event.symbol,
                                            &timeframe_str,
                                            vec![candle],
                                        )
                                        .await
                                    {
                                        tracing::warn!("Error processing market data for {} {}: {}", candle_event.symbol, timeframe_str, e);
                                    }
                                }
                                apex_protos::events::event::Payload::TickReceived(_tick) => {
                                    // Handle ticks if needed for faster reaction
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    });

    // Wait for shutdown signal or server exit
    tokio::select! {
        res = server_fut => {
            if let Err(e) = res {
                warn!("gRPC server exited with error: {:?}", e);
            }
        }
        _ = subscriber_fut => {
            warn!("Subscriber loop exited unexpectedly");
        }
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received, stopping...");
        }
    }

    info!("APEX Signal Engine stopped");
    Ok(())
}
