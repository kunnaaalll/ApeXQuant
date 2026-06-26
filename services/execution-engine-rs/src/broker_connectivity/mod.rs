use std::fmt::Debug;
use async_trait::async_trait;

pub mod mt5;
pub mod dxtrade;
pub mod ctrader;
pub mod fix;

pub use mt5::Mt5Adapter;
pub use dxtrade::DxTradeAdapter;
pub use ctrader::CTraderAdapter;
pub use fix::FixAdapter;

#[derive(Debug, Clone, PartialEq)]
pub enum BrokerError {
    ConnectionLost,
    AuthenticationFailed,
    Timeout,
    RateLimited,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountState {
    pub balance: f64,
    pub equity: f64,
    pub margin: f64,
    pub free_margin: f64,
    pub leverage: f64,
    pub drawdown: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionState {
    pub symbol: String,
    pub volume: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub profit: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderState {
    pub id: String,
    pub symbol: String,
    pub volume: f64,
    pub price: f64,
    pub is_open: bool,
}

#[async_trait]
pub trait BrokerAdapter: Send + Sync + Debug {
    async fn login(&mut self) -> Result<(), BrokerError>;
    async fn reconnect(&mut self) -> Result<(), BrokerError>;
    async fn heartbeat(&self) -> Result<(), BrokerError>;
    
    async fn sync_account(&self) -> Result<AccountState, BrokerError>;
    async fn sync_orders(&self) -> Result<Vec<OrderState>, BrokerError>;
    async fn sync_positions(&self) -> Result<Vec<PositionState>, BrokerError>;
}
