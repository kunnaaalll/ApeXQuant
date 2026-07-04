#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use analytics_engine::config::{AnalyticsConfig, ConfigError};
use analytics_engine::aggregation::AggregationStore;
use analytics_engine::trades::parse_trade_event;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("APEX V3 — Analytics Engine V1 Starting...");

    let config = AnalyticsConfig::from_env().map_err(|e| match e {
        ConfigError::MissingEnvVar(name) => {
            tracing::error!("Missing required env var: {}", name);
            format!("Missing env var: {}", name)
        }
        ConfigError::InvalidValue(name, msg) => {
            tracing::error!("Invalid config {}: {}", name, msg);
            format!("Invalid config {}: {}", name, msg)
        }
    })?;

    tracing::info!("Configuration loaded. Connecting to Redis at {}", config.redis_url);

    // Connect to Redis for event subscription
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .map_err(|e| format!("Redis client init failed: {}", e))?;

    // Shared aggregation store
    let store = Arc::new(Mutex::new(AggregationStore::new()));

    tracing::info!("Analytics Engine V1 running — consuming trade events...");

    // Main event loop: subscribe to completed_trades topic and process
    let store_clone = store.clone();
    tokio::spawn(async move {
        loop {
            match consume_trade_events(&redis_client, &store_clone).await {
                Ok(()) => {
                    tracing::info!("Trade event consumer completed gracefully");
                }
                Err(e) => {
                    tracing::error!("Trade event consumer error: {} — retrying in 5s", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Keep the service alive
    std::future::pending::<()>().await;
    Ok(())
}

async fn consume_trade_events(
    client: &redis::Client,
    store: &Arc<Mutex<AggregationStore>>,
) -> Result<(), String> {
    let mut conn = client
        .get_tokio_connection()
        .await
        .map_err(|e| format!("Redis connection failed: {}", e))?;

    // Subscribe to completed trades topic
    let mut pubsub = conn.into_pubsub();
    pubsub
        .subscribe("apex:events:trade_completed")
        .await
        .map_err(|e| format!("Redis SUBSCRIBE failed: {}", e))?;

    tracing::info!("Subscribed to apex:events:trade_completed");

    loop {
        use futures::StreamExt;
        let msg = pubsub
            .on_message()
            .next()
            .await
            .ok_or_else(|| "Redis subscription stream ended".to_string())?;

        let payload: Vec<u8> = msg
            .get_payload_bytes()
            .to_vec();

        match parse_trade_event(&payload) {
            Ok(trade) => {
                match store.lock() {
                    Ok(mut locked_store) => {
                        locked_store.ingest(&trade);
                        tracing::debug!(
                            "Ingested trade {} | strategy={} | pnl={}",
                            trade.trade_id,
                            trade.strategy_id,
                            trade.net_pnl
                        );
                    }
                    Err(e) => {
                        tracing::error!("Aggregation store lock poisoned: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse trade event: {}", e);
            }
        }
    }
}
