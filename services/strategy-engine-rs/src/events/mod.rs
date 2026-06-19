use rust_decimal::Decimal;
use crate::health::HealthState;
use crate::confidence::ConfidenceTier;
use crate::allocation::AllocationState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyEvent {
    StrategyCreated { id: String, timestamp: i64 },
    StrategyActivated { id: String, timestamp: i64 },
    StrategyPaused { id: String, reason: String, timestamp: i64 },
    StrategyRecovered { id: String, timestamp: i64 },
    StrategyRetired { id: String, reason: String, timestamp: i64 },
    HealthChanged { id: String, old: HealthState, new: HealthState, timestamp: i64 },
    ConfidenceChanged { id: String, old: ConfidenceTier, new: ConfidenceTier, timestamp: i64 },
    AllocationChanged { id: String, old: AllocationState, new: AllocationState, weight: Decimal, timestamp: i64 },
}
