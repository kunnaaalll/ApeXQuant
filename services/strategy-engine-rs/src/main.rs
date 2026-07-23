#![allow(warnings, clippy::all, deprecated)]
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tracing::{info, warn};

use strategy_engine_rs::api::server::start_server;
use strategy_engine_rs::api::service::StrategyState;
use strategy_engine_rs::event_bus_subscriber::EventBusSubscriber;
use strategy_engine_rs::meta::strategy_registry::StrategyRegistry;
use apex_protos::common::{OrderType, Price, Symbol, TradeSide, Volume};
use apex_protos::execution::{execution_engine_client::ExecutionEngineClient, ExecutionMode, ExecutionPreferences, NewOrder, SubmitOrderRequest, TimeInForce};
use apex_protos::events::event::Payload;
use rust_decimal::Decimal;
use std::time::Duration;

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
    let execution_url = env::var("EXECUTION_ENGINE_URL")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());
    let order_volume = env::var("DEFAULT_ORDER_VOLUME")
        .map_err(|e| format!("DEFAULT_ORDER_VOLUME must be configured: {e}"))?;

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
        let execution_url_for_task = execution_url.clone();
        tokio::spawn(async move {
            while let Some(event) = signal_rx.recv().await {
                if let Err(error) = submit_signal_order(&execution_url_for_task, &order_volume, event).await {
                    tracing::error!(%error, "Signal could not be submitted to execution engine");
                }
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

async fn submit_signal_order(
    execution_url: &str,
    configured_volume: &str,
    event: apex_protos::events::Event,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let signal = match event.payload {
        Some(Payload::SignalDetected(signal)) => signal,
        _ => return Err("signals.detected event did not contain SignalDetectedEvent".into()),
    };
    let entry = signal.entry_zone_high.as_ref().or(signal.entry_zone_low.as_ref())
        .ok_or("signal has no entry price")?;
    let stop_loss = signal.raw_detections.get("stop_loss").ok_or("signal has no stop loss")?;
    let take_profit = signal.raw_detections.get("take_profit").ok_or("signal has no take profit")?;
    let confidence = signal.raw_detections.get("confidence").ok_or("signal has no confidence metadata")?;
    let entry_value = Decimal::from_str_exact(&entry.value)?;
    let stop_value = Decimal::from_str_exact(stop_loss)?;
    let target_value = Decimal::from_str_exact(take_profit)?;
    let volume = Decimal::from_str_exact(configured_volume)?;
    if entry_value <= Decimal::ZERO || stop_value <= Decimal::ZERO || target_value <= Decimal::ZERO || volume <= Decimal::ZERO {
        return Err("signal contains non-positive order values".into());
    }
    let side = match TradeSide::try_from(signal.suggested_side)? {
        TradeSide::Buy | TradeSide::Sell => signal.suggested_side,
        _ => return Err("signal has unspecified trade side".into()),
    };
    let precision = entry.value.split('.').nth(1).map_or(0, str::len) as u32;
    let order = NewOrder {
        client_order_id: signal.signal_id.clone(),
        symbol: Some(Symbol { code: signal.symbol.clone(), exchange: String::new(), asset_class: 0, description: String::new() }),
        order_type: OrderType::Market as i32,
        side,
        volume: Some(Volume { units: volume.to_string(), lot_size: configured_volume.to_string(), fractional: true }),
        limit_price: Some(Price { value: entry_value.to_string(), digits: precision, currency: entry.currency.clone() }),
        stop_price: None,
        stop_loss: Some(Price { value: stop_value.to_string(), digits: precision, currency: entry.currency.clone() }),
        take_profit: Some(Price { value: target_value.to_string(), digits: precision, currency: entry.currency.clone() }),
        valid_until: signal.valid_until,
        time_in_force: TimeInForce::Ioc as i32,
        signal_id: signal.signal_id.clone(),
        strategy_id: signal.strategy_id.clone(),
        correlation_id: format!("signal:{}:confidence:{}", signal.signal_id, confidence),
        requester_service: "strategy-engine".to_string(),
    };
    let request = SubmitOrderRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        order: Some(order),
        preferences: Some(ExecutionPreferences { mode: ExecutionMode::Market as i32, preferred_broker: String::new(), use_smart_routing: true, slippage: None, require_post_trade_confirmation: true }),
        priority: 0,
    };

    let mut last_error = None;
    for attempt in 0..3 {
        match ExecutionEngineClient::connect(execution_url.to_string()).await {
            Ok(mut client) => match client.submit_order(tonic::Request::new(request.clone())).await {
                Ok(response) if response.get_ref().error_details.is_none() && response.get_ref().rejection_reason.is_empty() => {
                    info!(signal_id = %signal.signal_id, order_id = %response.get_ref().order_id, attempt, "Signal submitted to execution engine");
                    return Ok(());
                }
                Ok(response) => return Err(format!("execution rejected signal {}: {}", signal.signal_id, response.get_ref().rejection_reason).into()),
                Err(error) if matches!(error.code(), tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::ResourceExhausted) => last_error = Some(error.to_string()),
                Err(error) => return Err(error.into()),
            },
            Err(error) => last_error = Some(error.to_string()),
        }
        if attempt < 2 { tokio::time::sleep(Duration::from_millis(100_u64 * (attempt + 1))).await; }
    }
    Err(format!("transient execution failure after retries: {}", last_error.unwrap_or_else(|| "unknown".to_string())).into())
}
