use crate::api::server::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Serialize)]
pub enum ComponentState {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub postgres: ComponentState,
    pub redis: ComponentState,
    pub broker_registry: ComponentState,
    pub grpc_service: ComponentState,
    pub global_state: ComponentState,
}

// In a real implementation, this would check real components.
// For the deterministic requirement and adapter role, we mock this state.
static IS_READY: AtomicBool = AtomicBool::new(true);

pub fn set_ready(ready: bool) {
    IS_READY.store(ready, Ordering::SeqCst);
}

pub async fn readiness_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let global = if IS_READY.load(Ordering::SeqCst) {
        ComponentState::Ready
    } else {
        ComponentState::Unavailable
    };

    let mut postgres = ComponentState::Unavailable;
    if let Some(pg) = &app_state.pg_store {
        if sqlx::query("SELECT 1").execute(pg.pool()).await.is_ok() {
            postgres = ComponentState::Ready;
        }
    }

    let mut redis_state = ComponentState::Unavailable;
    if let Some(client) = &app_state.redis_client {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            let res: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;
            if res.is_ok() {
                redis_state = ComponentState::Ready;
            }
        }
    }

    let mut broker_registry = ComponentState::Unavailable;
    let mt5_health = app_state.mt5_adapter.heartbeat().await;
    let binance_health = app_state.binance_adapter.heartbeat().await;
    if mt5_health.is_ok() && binance_health.is_ok() {
        broker_registry = ComponentState::Ready;
    } else if mt5_health.is_ok() || binance_health.is_ok() {
        broker_registry = ComponentState::Degraded;
    }

    Json(ReadinessResponse {
        postgres,
        redis: redis_state,
        broker_registry,
        grpc_service: ComponentState::Ready,
        global_state: global,
    })
}
