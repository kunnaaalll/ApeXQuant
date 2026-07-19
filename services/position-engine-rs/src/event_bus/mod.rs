pub mod publisher;
pub mod subscriber;

pub use publisher::EventPublisher;
pub use subscriber::EventSubscriber;

use crate::positions::PositionState;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionEventPayload {
    pub position_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub state: PositionState,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    #[serde(with = "time::serde::rfc3339::option")]
    pub timestamp: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionHealthEventPayload {
    pub position_id: Uuid,
    pub health_score: f64,
    pub margin_utilization: f64,
    pub stop_distance: f64,
    pub drawdown: f64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub timestamp: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAnalyticsEventPayload {
    pub position_id: Uuid,
    pub holding_efficiency: f64,
    pub time_efficiency: f64,
    pub profit_velocity: f64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub timestamp: Option<OffsetDateTime>,
}
