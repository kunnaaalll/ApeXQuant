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

pub mod broker_connectivity;
pub mod connection_supervisor;
pub mod order_reconciliation;
pub mod position_recovery;
pub mod account_synchronization;
pub mod failover_engine;
pub mod disaster_recovery;
pub mod operational_governance;
pub mod event_bus;
pub mod event_bus_subscriber;

pub mod config;
pub mod broker_supervisor;

pub mod api;
#[cfg(test)]
pub mod tests;
