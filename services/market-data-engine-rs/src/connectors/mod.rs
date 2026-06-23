use crate::state_machine::ConnectionState;
use crate::health::FeedHealthGrade;
use crate::latency::LatencyGrade;
use crate::quality::FeedQuality;
use std::pin::Pin;
use std::future::Future;

pub mod mt5;
pub mod binance;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectorError {
    ConnectionFailed,
    DisconnectionFailed,
    InvalidState,
}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed => write!(f, "Connection Failed"),
            Self::DisconnectionFailed => write!(f, "Disconnection Failed"),
            Self::InvalidState => write!(f, "Invalid State Transition"),
        }
    }
}

impl std::error::Error for ConnectorError {}

pub trait MarketDataConnector: Send + Sync {
    fn connect(&mut self) -> BoxFuture<'_, Result<(), ConnectorError>>;
    fn disconnect(&mut self) -> BoxFuture<'_, Result<(), ConnectorError>>;
    
    fn health(&self) -> FeedHealthGrade;
    fn latency(&self) -> LatencyGrade;
    fn symbol_status(&self, symbol: &str) -> Result<ConnectionState, ConnectorError>;
    fn feed_quality(&self) -> FeedQuality;
    fn connection_state(&self) -> ConnectionState;
}
