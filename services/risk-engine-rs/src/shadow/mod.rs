use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub mod comparison;
pub mod drift;
pub mod reporter;
pub mod shadow_storage;
pub mod statistics;
pub mod validator;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyRiskState {
    pub drawdown: Decimal,
    pub exposure: Decimal,
    pub correlation: Decimal,
    pub hidden_leverage: Decimal,
    pub var: Decimal,
    pub expected_shortfall: Decimal,
    pub circuit_breakers_tripped: u32,
    pub recommendation_code: u32,
    pub stress_assessment: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RustRiskState {
    pub drawdown: Decimal,
    pub exposure: Decimal,
    pub correlation: Decimal,
    pub hidden_leverage: Decimal,
    pub var: Decimal,
    pub expected_shortfall: Decimal,
    pub circuit_breakers_tripped: u32,
    pub recommendation_code: u32,
    pub stress_assessment: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSnapshot {
    pub timestamp: DateTime<Utc>,
    pub legacy_state: LegacyRiskState,
    pub rust_state: RustRiskState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShadowEvent {
    ComparisonCreated,
    DriftDetected,
    MismatchDetected,
    WarningIssued,
}
