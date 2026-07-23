#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use apex_protos::learning::learning_engine_server::LearningEngineServer;
use apex_protos::events::{
    event_bus_service_client::EventBusServiceClient,
    event::Payload,
    stream_position, Event, StartPosition, StreamPosition, SubscribeRequest,
};
use learning_engine::api::server::GrpcLearningEngine;
use learning_engine::confidence::{ConfidenceEngine, ConfidenceMetrics};
use learning_engine::database::{LearningRepository, MemoryRepository};
use learning_engine::decay::{DecayMetrics, DecayTracker};
use learning_engine::metrics::LearningEngineMetrics;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;

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

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    });

    // Connect to Postgres
    tracing::info!("Connecting to PostgreSQL at {}...", database_url);
    let pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Postgres connection failed: {}", e))?;

    // Run Migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migrations failed: {}", e))?;

    tracing::info!("Migrations applied successfully");

    let learning_repo = Arc::new(LearningRepository::new(pool.clone()));
    let memory_repo = Arc::new(MemoryRepository::new(pool.clone()));

    let confidence_engine = Arc::new(ConfidenceEngine::new());
    let decay_tracker = Arc::new(DecayTracker::new());

    tracing::info!("Starting Event Bus consumer at {}", event_bus_url);
    let memory_repo_clone = memory_repo.clone();
    let learning_repo_clone = learning_repo.clone();
    let conf_clone = confidence_engine.clone();
    let decay_clone = decay_tracker.clone();

    tokio::spawn(async move {
        loop {
            match run_learning_loop(
                &event_bus_url,
                &memory_repo_clone,
                &learning_repo_clone,
                &conf_clone,
                &decay_clone,
            )
            .await
            {
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
    health_reporter
        .set_serving::<LearningEngineServer<GrpcLearningEngine>>()
        .await;

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
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!("Failed to install Ctrl+C handler: {}", e);
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install signal handler: {}", e);
                std::future::pending::<()>().await;
            }
        }
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
    event_bus_url: &str,
    memory_repo: &MemoryRepository,
    learning_repo: &LearningRepository,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) -> anyhow::Result<()> {
    let mut client = EventBusServiceClient::connect(event_bus_url.to_owned()).await?;
    let consumer_id = format!("learning-engine-{}", std::process::id());
    let request = SubscribeRequest {
        consumer_group: "learning-engine".to_string(),
        consumer_id: consumer_id.clone(),
        topics: vec![
            "trade.completed".to_string(),
            "trade.closed".to_string(),
            "execution.trade.closed".to_string(),
            "execution.position.closed".to_string(),
            "position.closed".to_string(),
            "position.opened".to_string(),
            "performance.metrics".to_string(),
            "risk.decision".to_string(),
            "portfolio.snapshot".to_string(),
        ],
        start_from: Some(StreamPosition {
            position: Some(stream_position::Position::From(StartPosition::Latest as i32)),
        }),
        max_batch_size: 100,
        max_wait_ms: None,
        filter: None,
    };
    let mut stream = client.subscribe(request).await?.into_inner();
    tracing::info!("Learning Engine subscribed through Event Bus (9 topics)");
    use futures::StreamExt;
    while let Some(batch) = stream.next().await {
        let batch = batch?;
        let mut ack_ids: Vec<String> = Vec::new();
        for event in batch.events {
            let event_id_str = event
                .event_id
                .as_ref()
                .and_then(|id| String::from_utf8(id.value.clone()).ok())
                .unwrap_or_default();

            let processed = process_event(&event, memory_repo, learning_repo, confidence_engine, decay_tracker).await;
            if processed {
                if !event_id_str.is_empty() {
                    ack_ids.push(event_id_str);
                }
            } else {
                tracing::warn!(
                    event_type = %event.event_type,
                    topic = %event.topic,
                    "Event processing failed — will not ack (bus will redeliver)"
                );
            }
        }
        if !ack_ids.is_empty() {
            if let Err(e) = client
                .ack(apex_protos::events::AckRequest {
                    consumer_group: "learning-engine".to_string(),
                    consumer_id: consumer_id.clone(),
                    event_ids: ack_ids,
                    failed: vec![],
                })
                .await
            {
                tracing::warn!("ACK failed: {}", e);
            }
        }
    }

    Err(anyhow::anyhow!("Event Bus subscription stream ended"))
}

/// Process a single event from any subscribed topic.
/// Returns true if the event was processed successfully (or skipped intentionally).
/// Returns false if a recoverable error occurred during persistence.
async fn process_event(
    event: &Event,
    memory_repo: &MemoryRepository,
    learning_repo: &LearningRepository,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) -> bool {
    let topic = event.topic.as_str();
    let strategy_id = event
        .correlation
        .as_ref()
        .and_then(|c| c.baggage.get("strategy_id").cloned())
        .unwrap_or_else(|| "unknown".to_string());

    match &event.payload {
        // ── PositionClosed — primary learning signal ─────────────────────────
        Some(Payload::PositionClosed(closed)) => {
            let net_pnl = closed
                .net_pnl
                .as_ref()
                .and_then(|m| m.amount.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);
            let gross_pnl = closed
                .gross_pnl
                .as_ref()
                .and_then(|m| m.amount.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);
            // PositionClosedEvent has no symbol field — extract from correlation baggage
            // or fall back to position_id as identifier
            let symbol = event
                .correlation
                .as_ref()
                .and_then(|c| c.baggage.get("symbol").cloned())
                .unwrap_or_else(|| closed.position_id.clone());
            let is_winner = net_pnl > Decimal::ZERO;

            LearningEngineMetrics::record_lesson_processed(is_winner);

            // Update strategy memory
            if !update_strategy_memory(
                &strategy_id,
                net_pnl,
                is_winner,
                memory_repo,
                confidence_engine,
                decay_tracker,
            )
            .await
            {
                return false;
            }

            // Persist to feature store
            let features = serde_json::json!({
                "net_pnl": net_pnl.to_string(),
                "gross_pnl": gross_pnl.to_string(),
                "symbol": symbol,
                "is_winner": is_winner,
                "strategy_id": strategy_id,
                "event_type": "PositionClosed",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "PositionClosed",
                    topic,
                    &strategy_id,
                    &symbol,
                    net_pnl,
                    gross_pnl,
                    Some(is_winner),
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist PositionClosed event: {}", e);
                return false;
            }

            tracing::info!(
                strategy_id = %strategy_id,
                symbol = %symbol,
                net_pnl = %net_pnl,
                is_winner = %is_winner,
                "PositionClosed processed"
            );
            true
        }

        // ── PositionOpened — track new position context ───────────────────────
        Some(Payload::PositionOpened(opened)) => {
            // PositionOpenedEvent.symbol is a String (not Option<String>)
            let symbol = opened.symbol.clone();
            let volume = opened
                .initial_volume
                .as_ref()
                .and_then(|v| v.units.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);

            let features = serde_json::json!({
                "symbol": symbol,
                "volume": volume.to_string(),
                "side": opened.side,
                "strategy_id": opened.strategy_id,
                "signal_id": opened.signal_id,
                "position_id": opened.position_id,
                "event_type": "PositionOpened",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "PositionOpened",
                    topic,
                    &strategy_id,
                    &symbol,
                    Decimal::ZERO,
                    Decimal::ZERO,
                    None,
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist PositionOpened event: {}", e);
                return false;
            }
            tracing::debug!(strategy_id = %strategy_id, symbol = %symbol, "PositionOpened persisted");
            true
        }

        // ── OrderFilled — trade completed confirmation ─────────────────────────
        // ExecutionOrderFilledEvent fields: order_id, execution_id, position_id,
        // fill_price, fill_volume, broker_execution_id, fill_time
        Some(Payload::OrderFilled(filled)) => {
            let fill_price = filled
                .fill_price
                .as_ref()
                .and_then(|p| p.value.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);
            let fill_volume = filled
                .fill_volume
                .as_ref()
                .and_then(|v| v.units.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);

            let features = serde_json::json!({
                "order_id": filled.order_id,
                "execution_id": filled.execution_id,
                "position_id": filled.position_id,
                "fill_price": fill_price.to_string(),
                "fill_volume": fill_volume.to_string(),
                "broker_execution_id": filled.broker_execution_id,
                "strategy_id": strategy_id,
                "event_type": "OrderFilled",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "OrderFilled",
                    topic,
                    &strategy_id,
                    "",  // symbol not available in this event; resolved via position_id
                    Decimal::ZERO,
                    Decimal::ZERO,
                    None,
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist OrderFilled event: {}", e);
                return false;
            }
            tracing::debug!(strategy_id = %strategy_id, "OrderFilled persisted");
            true
        }

        // ── RiskCheckPassed / RiskCheckFailed / RiskLimitBreached ─────────────
        // RiskCheckPassedEvent fields: check_id, order_id, passed_checks (Vec<String>)
        Some(Payload::RiskCheckPassed(passed)) => {
            let features = serde_json::json!({
                "check_id": passed.check_id,
                "order_id": passed.order_id,
                "passed_checks": passed.passed_checks,
                "strategy_id": strategy_id,
                "event_type": "RiskCheckPassed",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "RiskCheckPassed",
                    topic,
                    &strategy_id,
                    "",
                    Decimal::ZERO,
                    Decimal::ZERO,
                    Some(true),
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist RiskCheckPassed: {}", e);
                return false;
            }
            true
        }

        // RiskCheckFailedEvent fields: check_id, order_id, failures (Vec<RiskFailure>), hard_block
        Some(Payload::RiskCheckFailed(failed)) => {
            let failure_reasons: Vec<String> = failed
                .failures
                .iter()
                .map(|f| format!("{}: {}", f.check_name, f.reason))
                .collect();
            let features = serde_json::json!({
                "check_id": failed.check_id,
                "order_id": failed.order_id,
                "hard_block": failed.hard_block,
                "failure_reasons": failure_reasons,
                "strategy_id": strategy_id,
                "event_type": "RiskCheckFailed",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "RiskCheckFailed",
                    topic,
                    &strategy_id,
                    "",
                    Decimal::ZERO,
                    Decimal::ZERO,
                    Some(false),
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist RiskCheckFailed: {}", e);
                return false;
            }
            true
        }

        // RiskLimitBreachedEvent fields: limit_type, limit_id, current_value, limit_value,
        //   breach_amount, automatic_action_taken, action_taken
        Some(Payload::RiskLimitBreached(breached)) => {
            let features = serde_json::json!({
                "limit_type": breached.limit_type,
                "limit_id": breached.limit_id,
                "automatic_action_taken": breached.automatic_action_taken,
                "action_taken": breached.action_taken,
                "strategy_id": strategy_id,
                "event_type": "RiskLimitBreached",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "RiskLimitBreached",
                    topic,
                    &strategy_id,
                    "",
                    Decimal::ZERO,
                    Decimal::ZERO,
                    Some(false),
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist RiskLimitBreached: {}", e);
                return false;
            }
            true
        }

        // ── PortfolioRebalanced / AllocationUpdated — portfolio snapshot ───────
        Some(Payload::PortfolioRebalanced(rebalanced)) => {
            let features = serde_json::json!({
                "portfolio_id": rebalanced.portfolio_id,
                "strategy_id": strategy_id,
                "event_type": "PortfolioRebalanced",
                "total_value": rebalanced.total_value.as_ref().map(|v| v.amount.clone()),
            });
            if let Err(e) = learning_repo
                .record_event(
                    "PortfolioRebalanced",
                    topic,
                    &strategy_id,
                    "",
                    Decimal::ZERO,
                    Decimal::ZERO,
                    None,
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist PortfolioRebalanced: {}", e);
                return false;
            }
            true
        }

        // AllocationUpdatedEvent fields: portfolio_id, symbol, previous_weight, new_weight, reason
        Some(Payload::AllocationUpdated(alloc)) => {
            let prev_weight = alloc.previous_weight.as_ref().map(|p| p.value.clone());
            let new_weight = alloc.new_weight.as_ref().map(|p| p.value.clone());
            let features = serde_json::json!({
                "portfolio_id": alloc.portfolio_id,
                "symbol": alloc.symbol,
                "previous_weight": prev_weight,
                "new_weight": new_weight,
                "reason": alloc.reason,
                "strategy_id": strategy_id,
                "event_type": "AllocationUpdated",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "AllocationUpdated",
                    topic,
                    &strategy_id,
                    &alloc.symbol,
                    Decimal::ZERO,
                    Decimal::ZERO,
                    None,
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist AllocationUpdated: {}", e);
                return false;
            }
            true
        }

        // ── TrainingCompleted — model performance update ───────────────────────
        // TrainingCompletedEvent fields: training_run_id, model_id, duration,
        //   final_loss (Percentage), validation_accuracy (Percentage), improved
        Some(Payload::TrainingCompleted(training)) => {
            let final_loss = training.final_loss.as_ref().map(|p| p.value.clone());
            let validation_accuracy = training.validation_accuracy.as_ref().map(|p| p.value.clone());
            let features = serde_json::json!({
                "training_run_id": training.training_run_id,
                "model_id": training.model_id,
                "final_loss": final_loss,
                "validation_accuracy": validation_accuracy,
                "improved": training.improved,
                "strategy_id": strategy_id,
                "event_type": "TrainingCompleted",
            });
            if let Err(e) = learning_repo
                .record_event(
                    "TrainingCompleted",
                    topic,
                    &strategy_id,
                    "",
                    Decimal::ZERO,
                    Decimal::ZERO,
                    None,
                    features,
                )
                .await
            {
                tracing::error!("Failed to persist TrainingCompleted: {}", e);
                return false;
            }
            true
        }

        // ── All other payloads — log and ack (do not drop) ────────────────────
        Some(other_payload) => {
            // Events on subscribed topics that don't match our feature extraction
            // patterns are acknowledged but not stored — this is intentional.
            // We do NOT silently drop: we log at debug so it is visible in traces.
            tracing::debug!(
                topic = %topic,
                event_type = %event.event_type,
                payload_variant = ?std::mem::discriminant(other_payload),
                "Event acknowledged (no feature extraction for this payload type)"
            );
            true
        }

        // ── No payload ─────────────────────────────────────────────────────────
        None => {
            tracing::warn!(
                topic = %topic,
                event_type = %event.event_type,
                "Event received with no payload — acknowledging"
            );
            true
        }
    }
}

/// Update strategy memory with EMA/confidence/decay after a closed position.
/// Returns false if the DB operation fails.
async fn update_strategy_memory(
    strategy_id: &str,
    net_pnl: Decimal,
    is_winner: bool,
    memory_repo: &MemoryRepository,
    confidence_engine: &ConfidenceEngine,
    decay_tracker: &DecayTracker,
) -> bool {
    let current_memory = match memory_repo.get_memory(strategy_id).await {
        Ok(Some(row)) => row,
        Ok(None) => learning_engine::database::StrategyMemoryRow {
            total_trades: 0,
            winning_trades: 0,
            regime_quality: Decimal::ONE,
            execution_quality: Decimal::ONE,
            ema_return: Decimal::ZERO,
            historical_sum_return: Decimal::ZERO,
            regime_transitions: 0,
        },
        Err(e) => {
            tracing::error!("Failed to fetch memory for {}: {}", strategy_id, e);
            return false;
        }
    };

    let new_ema = decay_tracker.update_ema(current_memory.ema_return, net_pnl);

    let historical_mean = if current_memory.total_trades > 0 {
        current_memory.historical_sum_return / Decimal::from(current_memory.total_trades)
    } else {
        Decimal::ZERO
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
        Decimal::new(5, 1) // 0.5 default
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

    if let Err(e) = memory_repo
        .update_memory(
            strategy_id,
            net_pnl,
            is_winner,
            Decimal::ZERO, // regime_delta — updated by regime events
            Decimal::ZERO, // exec_delta — updated by execution events
            new_ema,
        )
        .await
    {
        tracing::error!("Failed to update memory for {}: {}", strategy_id, e);
        return false;
    }

    tracing::info!(
        strategy_id = %strategy_id,
        n = current_memory.total_trades + 1,
        confidence = %conf_output.composite_score,
        decay = %decay_output.decay_score,
        urgency = %decay_output.urgency_score,
        "Strategy memory updated"
    );

    use rust_decimal::prelude::ToPrimitive;
    let urgency_f = decay_output.urgency_score.to_f64().unwrap_or(0.0);
    if urgency_f > 0.8 {
        tracing::warn!(
            strategy_id = %strategy_id,
            "Strategy urgency > 0.80 — flagging for retraining"
        );
    }

    true
}
