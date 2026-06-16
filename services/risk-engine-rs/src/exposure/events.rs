use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::exposure::exposure_state::RiskState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PositionOpenedEvent {
    pub symbol: String,
    pub amount: Decimal,
    pub is_long: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PositionClosedEvent {
    pub symbol: String,
    pub amount: Decimal,
    pub is_long: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExposureUpdatedEvent {
    pub symbol: String,
    pub new_gross_exposure: Decimal,
    pub new_net_exposure: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConcentrationChangedEvent {
    pub new_concentration_score: Decimal,
    pub new_diversification_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterDetectedEvent {
    pub cluster_type: String, // e.g., "USD_Short", "Tech_Sector"
    pub cluster_weight: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskStateChangedEvent {
    pub old_state: RiskState,
    pub new_state: RiskState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExposureRiskEvent {
    PositionOpened(PositionOpenedEvent),
    PositionClosed(PositionClosedEvent),
    ExposureUpdated(ExposureUpdatedEvent),
    ConcentrationChanged(ConcentrationChangedEvent),
    ClusterDetected(ClusterDetectedEvent),
    RiskStateChanged(RiskStateChangedEvent),
}
