#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod attribution;
pub mod bus;
pub mod confidence;
pub mod decay;
pub mod discovery;
pub mod memory;
pub mod pattern;
pub mod recommendation;
pub mod reinforcement;
pub mod replay;

// Phase 2 modules
pub mod adaptation;
pub mod anomaly;
pub mod api;
pub mod certification;
pub mod clustering;
pub mod database;
pub mod drift;
pub mod event_bus;
pub mod explanation;
pub mod metrics;
pub mod optimization;
pub mod promotion;
pub mod regime_memory;
pub mod retirement;
