use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::models::{AllocationSnapshot, AllocationState, CapitalAllocationDecision};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllocationEventType {
    StateTransition { from: AllocationState, to: AllocationState },
    ReserveUpdated { total_reserved: Decimal },
    TradeAdmitted { symbol: String, decision: CapitalAllocationDecision },
    TradeRejected { symbol: String, decision: CapitalAllocationDecision },
    CapacityAdjusted,
    DrawdownStateChanged,
    HeatStateChanged,
    CircuitBreakerActivated,
    RecoveryDecayed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllocationEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub event_type: AllocationEventType,
    pub reason: String,
    pub snapshot: AllocationSnapshot,
}
