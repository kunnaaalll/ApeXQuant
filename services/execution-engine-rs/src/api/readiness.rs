use axum::{response::IntoResponse, Json};
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

pub async fn readiness_handler() -> impl IntoResponse {
    let state = if IS_READY.load(Ordering::SeqCst) {
        ComponentState::Ready
    } else {
        ComponentState::Unavailable
    } ;

    Json(ReadinessResponse {
        postgres: ComponentState::Ready,
        redis: ComponentState::Ready,
        broker_registry: ComponentState::Ready,
        grpc_service: ComponentState::Ready,
        global_state: state,
    })
}
