#![deny(unsafe_code)]

pub mod brokers;
pub mod events;
pub mod execution;
pub mod fills;
pub mod liquidity;
pub mod order;
pub mod order_split;
pub mod policies;
pub mod position;
pub mod slippage;
pub mod snapshots;
pub mod state;
pub mod validation;
pub mod microstructure;
pub mod market;
pub mod latency;
pub mod execution_cost;
pub mod execution_risk;
pub mod shadow;
pub mod storage;

pub mod api;
#[cfg(test)]
pub mod tests;
