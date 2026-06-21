#![deny(unsafe_code)]

pub mod broker;
pub mod capabilities;
pub mod connection;
pub mod errors;
pub mod events;
pub mod health;
pub mod requests;
pub mod responses;
pub mod snapshots;

pub mod binance;
pub mod mt5;
pub mod registry;

pub use broker::BrokerAdapter;
pub use connection::ConnectionState;
pub use errors::BrokerError;
pub use health::BrokerHealth;
