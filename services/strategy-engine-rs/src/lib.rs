#![deny(unsafe_code)]
#![warn(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used
)]

pub mod adaptive;
pub mod adequacy;
pub mod allocation;
pub mod analytics;
pub mod clustering;
pub mod discovery;
pub mod optimizer;
pub mod recommendations;
pub mod api;
pub mod confidence;
pub mod context;
pub mod degradation;
pub mod drift;
pub mod events;
pub mod evidence;
pub mod health;
pub mod intelligence;
pub mod learning;
pub mod lifecycle;
pub mod memory;
pub mod orchestration;
pub mod pattern;
pub mod ranking;
pub mod recovery;
pub mod regime;
pub mod session;
pub mod shadow;
pub mod snapshots;
pub mod state;
pub mod storage;
pub mod strategy;
pub mod streaks;
pub mod symbol;
pub mod timeframe;
pub mod validation;

// Phase 5 modules
pub mod counterfactual;
pub mod evolution;
pub mod meta;
pub mod overfitting;
pub mod research;
pub mod simulator;

#[cfg(test)]
mod tests;
