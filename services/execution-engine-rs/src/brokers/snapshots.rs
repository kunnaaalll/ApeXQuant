use super::health::BrokerHealth;
use super::connection::ConnectionState;
use super::responses::{AccountInfo, OpenPosition, PendingOrder, SymbolInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrokerSnapshot {
    pub broker_id: String,
    pub connection_state: ConnectionState,
    pub health: BrokerHealth,
    pub account: Option<AccountInfo>,
    pub positions: Vec<PositionSnapshot>,
    pub orders: Vec<PendingOrder>,
    pub symbols: Vec<SymbolInfo>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PositionSnapshot {
    pub position: OpenPosition,
    pub last_update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionSnapshot {
    pub broker_id: String,
    pub state: ConnectionState,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub broker_id: String,
    pub health: BrokerHealth,
    pub timestamp: i64,
}
