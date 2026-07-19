//! Analytics Engine — Trade Consumer and Performance Analytics
//!
//! Provides all modules for trade consumption, PnL aggregation,
//! performance metrics computation, and event bus publication.

pub mod aggregation;
pub mod config;
pub mod error;
pub mod metrics;
pub mod publisher;
pub mod statistics;
pub mod trades;
