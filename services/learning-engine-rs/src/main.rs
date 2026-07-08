#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use learning_engine::confidence::{ConfidenceEngine, ConfidenceMetrics};
use learning_engine::decay::{DecayTracker, DecayMetrics};
use learning_engine::database::{LearningRepository, MemoryRepository};
use learning_engine::metrics::LearningEngineMetrics;
use learning_engine::api::server::GrpcLearningEngine;
use apex_protos::learning::learning_engine_server::LearningEngineServer;
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use serde_json::Value;
use sqlx::PgPool;
use tonic::transport::Server;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("APEX V3 Learning Engine — Starting...");

    // Setup Prometheus Exporter
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9005))
        .install()
        .map_err(|e| anyhow::anyhow!("Prometheus setup failed: {}", e))?;

    let event_bus_url = std::env::var("EVENTBUS_URL")
        .or_else(|_| std::env::var("EVENT_BUS_URL"))
        .unwrap_or_else(|_| "http://localhost:50050".to_string());

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/apex_learning".to_string());

    // Connect to Postgres
    tracing::info!("Connecting to PostgreSQL at {}...", database_url);
    let pool = PgPool::connect(&database_url).await
        .map_err(|e| anyhow::anyhow!("Postgres connection failed: {}", e))?;

    // Run Migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migrations failed: {}", e))?;

    let learning_repo = Arc::new(LearningRepository::new(pool.clone()));
    let memory_repo = Arc::new(MemoryRepository::new(pool.clone()));

    // Initialize Event Bus (Publishing)
    let _bus = learning_engine::bus::EventBusIntegration::new();
    match learning_engine::event_bus::EventBusPublisher::connect(event_bus_url.clone()).await {
        Ok(_publisher) => {
            tracing::info!("Event Bus publisher connected at {}", event_bus_url);
        }
        Err(e) => {
            tracing::warn!("Failed to connect to Event Bus publisher at {}: {}", event_bus_url, e);
        }
    }

    let confidence_engine = Arc::new(ConfidenceEngine::new());
    let decay_tracker = Arc::new(DecayTracker::new());

    // Connect to Redis for event subscription
    let redis_client = redis::Client::open(redis_url.as_str())
        .map_err(|e| anyhow::anyhow!("Redis client init failed: {}", e))?;

    tracing::info!("Starting Redis Event Loop...");
    let redis_client_clone = Arc::new(redis_client);
    let memory_repo_clone = memory_repo.clone();
    let conf_clone = confidence_engine.clone();
    let decay_clone = decay_tracker.clone();

    tokio::spawn(async move {
        loop {
            match run_learning_loop(&redis_client_clone, &memory_repo_clone, &conf_clone, &decay_clone).await {
                Ok(()) => {
                    tracing::info!("Learning loop completed gracefully");
                    break;
                }
                Err(e) => {
                    tracing::error!("Learning loop error: {} — reconnecting in 5s", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    let grpc_addr = "0.0.0.0:50054".parse()?;
    tracing::info!("Learning Engine gRPC listening on {}", grpc_addr);

    let learning_service = GrpcLearningEngine::new(learning_repo);
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<LearningEngineServer<GrpcLearningEngine>>().await;

    Server::builder()
        .add_service(health_service)
        .add_service(LearningEngineServer::new(learning_service))
        .serve_with_shutdown(grpc_addr, shutdown_signal())
        .await?;

    tracing::info!("Learning Engine shut down gracefully");

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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

async fn run_learning_loop(
    redis_client: &redis::Client,
    memory_repo: &MemoryRepository,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) -> anyhow::Result<()> {
    let mut pubsub = redis_client.get_async_pubsub().await
        .map_err(|e| anyhow::anyhow!("Redis pubsub connection failed: {}", e))?;
    pubsub.subscribe("apex:events:trade_completed").await
        .map_err(|e| anyhow::anyhow!("Subscribe failed: {}", e))?;

    tracing::info!("Learning Engine subscribed to apex:events:trade_completed");

    use futures::StreamExt;
    while let Some(msg) = pubsub.on_message().next().await {
        let payload_bytes: Vec<u8> = msg.get_payload_bytes().to_vec();

        if let Ok(event) = serde_json::from_slice::<Value>(&payload_bytes) {
            process_trade_event(&event, memory_repo, confidence_engine, decay_tracker).await;
        } else {
            tracing::warn!("Failed to parse trade event JSON");
        }
    }

    Err(anyhow::anyhow!("Redis subscription stream ended"))
}

async fn process_trade_event(
    event: &Value,
    memory_repo: &MemoryRepository,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) {
    let strategy_id = match event["strategy_id"].as_str() {
        Some(s) => s.to_string(),
        None => return,
    };
    let net_pnl = event["net_pnl"].as_str()
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::new(0, 0));
    let is_winner = net_pnl > Decimal::new(0, 0);

    LearningEngineMetrics::record_lesson_processed(is_winner);

    let current_memory = match memory_repo.get_memory(&strategy_id).await {
        Ok(Some(row)) => row,
        Ok(None) => learning_engine::database::StrategyMemoryRow {
            total_trades: 0,
            winning_trades: 0,
            regime_quality: Decimal::ONE,
            execution_quality: Decimal::ONE,
            ema_return: Decimal::new(0, 0),
            historical_sum_return: Decimal::new(0, 0),
            regime_transitions: 0,
        },
        Err(e) => {
            tracing::error!("Failed to fetch memory for {}: {}", strategy_id, e);
            return;
        }
    };

    let new_ema = decay_tracker.update_ema(current_memory.ema_return, net_pnl);

    let historical_mean = if current_memory.total_trades > 0 {
        current_memory.historical_sum_return / Decimal::from(current_memory.total_trades)
    } else {
        Decimal::new(0, 0)
    };

    let conf_metrics = ConfidenceMetrics {
        total_trades: current_memory.total_trades as u64 + 1,
        winning_trades: current_memory.winning_trades as u64 + if is_winner { 1 } else { 0 },
        regime_quality: current_memory.regime_quality,
        execution_quality: current_memory.execution_quality,
    };
    let conf_output = confidence_engine.compute_confidence(&conf_metrics);

    let baseline_win_rate = if current_memory.total_trades > 10 {
        Decimal::from(current_memory.winning_trades) / Decimal::from(current_memory.total_trades)
    } else {
        Decimal::new(5, 1)
    };

    let decay_metrics = DecayMetrics {
        ema_return: new_ema,
        historical_mean_return: historical_mean,
        current_win_rate: conf_output.bayesian_win_rate,
        baseline_win_rate,
        regime_quality: current_memory.regime_quality,
        regime_transitions: current_memory.regime_transitions as u32,
    };
    let decay_output = decay_tracker.compute_decay(&decay_metrics);

    let regime_delta = event["regime_delta"].as_str()
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::new(0, 0));
    let exec_delta = event["execution_delta"].as_str()
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::new(0, 0));

    // Write back to repository
    if let Err(e) = memory_repo.update_memory(
        &strategy_id,
        net_pnl,
        is_winner,
        regime_delta,
        exec_delta,
        new_ema
    ).await {
        tracing::error!("Failed to update memory for {}: {}", strategy_id, e);
    }

    tracing::info!(
        strategy_id = %strategy_id,
        n = current_memory.total_trades + 1,
        confidence = %conf_output.composite_score,
        decay = %decay_output.decay_score,
        urgency = %decay_output.urgency_score,
        "Learning engine update"
    );

    let urgency_f = decay_output.urgency_score.to_f64_retain().unwrap_or(0.0);
    if urgency_f > 0.8 {
        tracing::warn!(
            strategy_id = %strategy_id,
            "Strategy urgency > 0.80 — flagging for retraining"
        );
    }
}

trait ToF64Retain {
    fn to_f64_retain(&self) -> Option<f64>;
}
impl ToF64Retain for Decimal {
    fn to_f64_retain(&self) -> Option<f64> {
        use rust_decimal::prelude::ToPrimitive;
        self.to_f64()
    }
}
