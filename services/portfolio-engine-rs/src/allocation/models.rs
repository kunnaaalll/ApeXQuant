use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationState {
    Aggressive,
    Normal,
    Defensive,
    Conservative,
    Recovery,
    Frozen,
}

impl AllocationState {
    pub fn is_frozen(&self) -> bool {
        matches!(self, Self::Frozen)
    }

    pub fn can_transition_to(&self, new_state: &Self) -> bool {
        // Prevent skipping directly from Frozen to Aggressive, etc.
        match (self, new_state) {
            (Self::Frozen, Self::Recovery) => true,
            (Self::Frozen, _) => false,
            (Self::Recovery, Self::Frozen) => true,
            (Self::Recovery, Self::Defensive) | (Self::Recovery, Self::Conservative) => true,
            (Self::Recovery, Self::Aggressive) | (Self::Recovery, Self::Normal) => false,
            _ => true, // Normal transitions between others
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeAdmissionDecision {
    Approve,
    ApproveReduced,
    Delay,
    Reject,
    Freeze,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdmissionFactor {
    pub name: String,
    pub description: String,
    pub blocks_admission: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapitalAllocationDecision {
    pub can_accept_trade: bool,
    pub admission_decision: TradeAdmissionDecision,
    pub allocation_size: Decimal,
    pub remaining_capacity: Decimal,
    pub reserved_capacity: Decimal,
    pub emergency_capacity: Decimal,
    pub reason: String,
    pub contributing_factors: Vec<AdmissionFactor>,
    pub heat_contribution: u8,
    pub exposure_contribution: Decimal,
    pub confidence: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapitalCapacity {
    pub available_capital: Decimal,
    pub available_risk: Decimal,
    pub capital_utilization: Decimal,
    pub risk_utilization: Decimal,
    pub exposure_utilization: Decimal,
    pub margin_utilization: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllocationSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub state: AllocationState,
    pub capacity: CapitalCapacity,
    pub reserved_capital: Decimal,
    pub emergency_reserve: Decimal,
}
