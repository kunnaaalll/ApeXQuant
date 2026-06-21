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

#[cfg(test)]
pub mod tests;
