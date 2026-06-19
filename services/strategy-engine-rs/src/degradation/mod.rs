pub mod collapse_detector;
pub mod edge_decay;
pub mod events;
pub mod snapshot;
pub mod strategy_degradation;

pub use strategy_degradation::{DegradationEngine, DegradationState};

#[cfg(test)]
mod tests;
