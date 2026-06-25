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
pub mod promotion;
pub mod retirement;
pub mod optimization;
pub mod clustering;
pub mod regime_memory;
pub mod drift;
pub mod anomaly;
pub mod explanation;
pub mod certification;
