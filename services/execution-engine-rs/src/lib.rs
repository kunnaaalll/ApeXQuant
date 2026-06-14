pub mod api;
pub mod execution;
pub mod health;
pub mod metrics;
pub mod orders;
pub mod reconciliation;
pub mod retry;
pub mod state_machine;
pub mod trade_management;

/// Configuration for the execution engine.
pub mod config;
/// Storage and persistence logic.
pub mod storage;
