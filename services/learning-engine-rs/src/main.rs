#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use learning_engine::confidence::{ConfidenceEngine, ConfidenceMetrics};
use learning_engine::decay::{DecayTracker, DecayMetrics};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde_json::Value;

/// Per-strategy running state for learning engine.
#[derive(Debug, Clone)]
struct StrategyState {
    total_trades: u64,
    winning_trades: u64,
    regime_quality: Decimal,
    execution_quality: Decimal,
    ema_return: Decimal,
    historical_sum_return: Decimal,
    regime_transitions: u32,
}

impl Default for StrategyState {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            regime_quality: Decimal::ONE,
            execution_quality: Decimal::ONE,
            ema_return: Decimal::ZERO,
            historical_sum_return: Decimal::ZERO,
            regime_transitions: 0,
        }
    }
}

type StrategyMap = Arc<Mutex<HashMap<String, StrategyState>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("APEX V3 Learning Engine — Starting...");

    let event_bus_url = std::env::var("EVENTBUS_URL")
        .or_else(|_| std::env::var("EVENT_BUS_URL"))
        .unwrap_or_else(|_| "http://localhost:50050".to_string());

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Initialize event bus integration
    let _bus = learning_engine::bus::EventBusIntegration::new();

    match learning_engine::event_bus::EventBusPublisher::connect(event_bus_url.clone()).await {
        Ok(_publisher) => {
            tracing::info!("Event Bus connected at {}", event_bus_url);
        }
        Err(e) => {
            tracing::warn!("Failed to connect to Event Bus at {}: {}", event_bus_url, e);
        }
    }

    // Shared strategy states
    let strategy_states: StrategyMap = Arc::new(Mutex::new(HashMap::new()));

    let confidence_engine = Arc::new(ConfidenceEngine::new());
    let decay_tracker = Arc::new(DecayTracker::new());

    // Connect to Redis for event subscription
    let redis_client = match redis::Client::open(redis_url.as_str()) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to create Redis client: {}", e);
            return Err(anyhow::anyhow!("Redis client init failed: {}", e));
        }
    };

    tracing::info!("Learning Engine running — consuming trade events from Redis...");

    // Continuous event loop with reconnect
    loop {
        let states = strategy_states.clone();
        let conf = confidence_engine.clone();
        let decay = decay_tracker.clone();
        let client = redis_client.clone();

        match run_learning_loop(&client, &states, &conf, &decay).await {
            Ok(()) => {
                tracing::info!("Learning loop completed gracefully");
            }
            Err(e) => {
                tracing::error!("Learning loop error: {} — reconnecting in 5s", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn run_learning_loop(
    redis_client: &redis::Client,
    states: &StrategyMap,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) -> anyhow::Result<()> {
    let mut conn = redis_client.get_tokio_connection().await
        .map_err(|e| anyhow::anyhow!("Redis connection failed: {}", e))?;

    let mut pubsub = conn.into_pubsub();
    pubsub.subscribe("apex:events:trade_completed").await
        .map_err(|e| anyhow::anyhow!("Subscribe failed: {}", e))?;

    tracing::info!("Learning Engine subscribed to apex:events:trade_completed");

    loop {
        use futures::StreamExt;
        let msg = pubsub.on_message().next().await
            .ok_or_else(|| anyhow::anyhow!("Redis subscription stream ended"))?;

        let payload_bytes: Vec<u8> = msg.get_payload_bytes().to_vec();

        if let Ok(event) = serde_json::from_slice::<Value>(&payload_bytes) {
            process_trade_event(&event, states, confidence_engine, decay_tracker);
        } else {
            tracing::warn!("Failed to parse trade event JSON");
        }
    }
}

fn process_trade_event(
    event: &Value,
    states: &StrategyMap,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) {
    let strategy_id = match event["strategy_id"].as_str() {
        Some(s) => s.to_string(),
        None => return,
    };
    let net_pnl = event["net_pnl"].as_str()
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);
    let is_winner = net_pnl > Decimal::ZERO;

    match states.lock() {
        Ok(mut locked) => {
            let state = locked.entry(strategy_id.clone()).or_default();
            state.total_trades += 1;
            if is_winner { state.winning_trades += 1; }
            state.historical_sum_return += net_pnl;

            // Update EMA return
            state.ema_return = decay_tracker.update_ema(state.ema_return, net_pnl);

            let historical_mean = if state.total_trades > 0 {
                state.historical_sum_return / Decimal::from(state.total_trades)
            } else {
                Decimal::ZERO
            };

            // Compute confidence
            let conf_metrics = ConfidenceMetrics {
                total_trades: state.total_trades,
                winning_trades: state.winning_trades,
                regime_quality: state.regime_quality,
                execution_quality: state.execution_quality,
            };
            let conf_output = confidence_engine.compute_confidence(&conf_metrics);

            // Compute decay
            let baseline_win_rate = if state.total_trades > 10 {
                Decimal::from(state.winning_trades) / Decimal::from(state.total_trades)
            } else {
                Decimal::new(5, 1) // Use 0.5 prior for small samples
            };

            let decay_metrics = DecayMetrics {
                ema_return: state.ema_return,
                historical_mean_return: historical_mean,
                current_win_rate: conf_output.bayesian_win_rate,
                baseline_win_rate,
                regime_quality: state.regime_quality,
                regime_transitions: state.regime_transitions,
            };
            let decay_output = decay_tracker.compute_decay(&decay_metrics);

            tracing::info!(
                strategy_id = %strategy_id,
                n = state.total_trades,
                confidence = %conf_output.composite_score,
                decay = %decay_output.decay_score,
                urgency = %decay_output.urgency_score,
                "Learning engine update"
            );

            // Emit retrain alert if urgency > 0.8
            let urgency_f = decay_output.urgency_score.to_f64_retain().unwrap_or(0.0);
            if urgency_f > 0.8 {
                tracing::warn!(
                    strategy_id = %strategy_id,
                    "Strategy urgency > 0.80 — flagging for retraining"
                );
            }
        }
        Err(e) => {
            tracing::error!("Strategy state lock poisoned: {}", e);
        }
    }
}

// Trait to get f64 from Decimal without feature flag in main
trait ToF64Retain {
    fn to_f64_retain(&self) -> Option<f64>;
}
impl ToF64Retain for Decimal {
    fn to_f64_retain(&self) -> Option<f64> {
        use rust_decimal::prelude::ToPrimitive;
        self.to_f64()
    }
}
