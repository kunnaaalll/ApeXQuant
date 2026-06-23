#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod aggregation;
pub mod anomaly;
pub mod binance_stream;
pub mod buffer;
pub mod candle;
pub mod confidence;
pub mod connectors;
pub mod depth;
pub mod dispatcher;
pub mod efficiency;
pub mod events;
pub mod failover;
pub mod feed;
pub mod gaps;
pub mod health;
pub mod imbalance;
pub mod latency;
pub mod liquidity;
pub mod market_state;
pub mod microstructure;
pub mod momentum;
pub mod mt5_stream;
pub mod noise;
pub mod normalization;
pub mod quality;
pub mod regime;
pub mod registry;
pub mod replay;
pub mod router;
pub mod scoring;
pub mod sequence;
pub mod session;
pub mod snapshots;
pub mod spread;
pub mod state_machine;
pub mod statistics;
pub mod storage;
pub mod streaming;
pub mod symbol;
pub mod throughput;
pub mod tick;
pub mod trend;
pub mod validation;
pub mod volatility;
pub mod websocket;

#[cfg(test)]
mod tests;
