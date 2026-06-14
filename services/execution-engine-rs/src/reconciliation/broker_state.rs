use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerState {
    pub positions: Vec<BrokerPosition>,
    pub active_orders: Vec<BrokerOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerPosition {
    pub symbol: String,
    pub volume: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerOrder {
    pub id: String,
    pub status: String,
}
