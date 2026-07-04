use crate::connection_supervisor::ConnectionState;
use crate::brokers::broker::{AccountState, PositionState, OrderState};

#[derive(Debug, Clone)]
pub struct BrokerSnapshot {
    pub account_state: AccountState,
    pub positions: Vec<PositionState>,
    pub orders: Vec<OrderState>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct RecoverySnapshot {
    pub positions: Vec<PositionState>,
    pub orders: Vec<OrderState>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectivitySnapshot {
    pub state: ConnectionState,
    pub reconnect_attempts: u32,
    pub auth_status: bool,
    pub timestamp: u64,
}
