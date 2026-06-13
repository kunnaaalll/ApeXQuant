//! gRPC API implementation

pub mod server;
pub mod service;
pub mod interceptors;

pub use server::start_grpc_server;
pub use service::SignalEngineService;

use crate::SignalEngine;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared API state
#[derive(Clone)]
pub struct ApiState {
    /// Signal engine instance
    pub engine: Arc<RwLock<SignalEngine>>,
}
