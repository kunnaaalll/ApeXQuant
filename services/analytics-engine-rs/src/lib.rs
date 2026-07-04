//! Analytics Engine — Trade Consumer and Performance Analytics
//!
//! Provides all modules for trade consumption, PnL aggregation,
//! performance metrics computation, and event bus publication.

pub mod config;
pub mod trades;
pub mod aggregation;
pub mod metrics;
pub mod statistics;
pub mod publisher;
pub mod error;
