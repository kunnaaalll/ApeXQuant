#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod attribution;
pub mod event_bus;
pub mod execution_model;
pub mod market_replay;
pub mod monte_carlo;
pub mod optimization;
pub mod performance;
pub mod portfolio_simulation;
pub mod risk_simulation;
pub mod simulation;
pub mod storage;
pub mod strategy_simulation;
