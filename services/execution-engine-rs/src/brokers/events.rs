use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BrokerEvent {
    Connection(ConnectionEvent),
    Health(HealthEvent),
    Fill(FillEvent),
    Order(OrderEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionEvent {
    pub broker_id: String,
    pub previous_state: String,
    pub new_state: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthEvent {
    pub broker_id: String,
    pub latency_ms: Decimal,
    pub uptime_percentage: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FillEvent {
    pub broker_id: String,
    pub order_id: String,
    pub fill_price: Decimal,
    pub fill_volume: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderEvent {
    pub broker_id: String,
    pub order_id: String,
    pub status: String,
    pub timestamp: i64,
}
