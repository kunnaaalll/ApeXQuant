use rust_decimal::Decimal;
use crate::state::StrategyState;
use crate::health::HealthState;
use crate::confidence::ConfidenceTier;
use crate::allocation::AllocationState;
use crate::degradation::DegradationState;
use crate::lifecycle::LifecycleState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategySnapshot {
    pub strategy_id: String,
    pub timestamp: i64,
    pub current_state: StrategyState,
    pub health: HealthState,
    pub confidence: ConfidenceTier,
    pub allocation: AllocationState,
    pub degradation: DegradationState,
    pub lifecycle: LifecycleState,
    pub overall_score: Decimal,
}
