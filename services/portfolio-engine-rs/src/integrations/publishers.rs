use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationChangeEvent {
    pub account_id: String,
    pub symbol: String,
    pub new_allocation: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureLimitEvent {
    pub account_id: String,
    pub current_exposure: Decimal,
    pub limit: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalRotationEvent {
    pub from_symbol: String,
    pub to_symbol: String,
    pub amount: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioWarningEvent {
    pub account_id: String,
    pub warning_type: String,
    pub message: String,
    pub timestamp: i64,
}
