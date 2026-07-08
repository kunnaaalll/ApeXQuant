#![deny(unsafe_code)]

pub mod brokers;
pub mod events;
pub mod execution;
pub mod execution_cost;
pub mod execution_risk;
pub mod fills;
pub mod latency;
pub mod liquidity;
pub mod market;
pub mod microstructure;
pub mod order;
pub mod order_split;
pub mod policies;
pub mod position;
pub mod shadow;
pub mod slippage;
pub mod snapshots;
pub mod state;
pub mod storage;
pub mod validation;

pub mod account_synchronization;
pub mod connection_supervisor;
pub mod disaster_recovery;
pub mod event_bus;
pub mod event_bus_subscriber;
pub mod failover_engine;
pub mod operational_governance;
pub mod order_reconciliation;
pub mod position_recovery;

pub mod broker_supervisor;
pub mod config;

pub mod api;
#[cfg(test)]
pub mod tests;
