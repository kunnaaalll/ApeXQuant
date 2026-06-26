use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllocationAction {
    CapitalIncrease(Decimal),
    CapitalReduction(Decimal),
    AccountRotation { from_account: Uuid, to_account: Uuid },
    StrategyScaling(Decimal),
    ExposureReduction(Decimal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllocationConstraints {
    pub portfolio_heat_limit: Decimal,
    pub max_drawdown_limit: Decimal,
    pub correlation_cluster_max_exposure: Decimal,
    pub prop_firm_rules_satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllocationDecision {
    pub decision_id: Uuid,
    pub target_id: Uuid,
    pub action: AllocationAction,
    pub constraints_applied: AllocationConstraints,
    pub is_approved: bool, // Based on constraints
    pub generated_at: OffsetDateTime,
}

impl AllocationDecision {
    pub fn new(
        target_id: Uuid,
        action: AllocationAction,
        constraints_applied: AllocationConstraints,
    ) -> Self {
        // Validation logic based on constraints
        let mut is_approved = constraints_applied.prop_firm_rules_satisfied;
        
        // Simple mock validation rules
        if constraints_applied.portfolio_heat_limit < Decimal::ZERO {
            is_approved = false;
        }
        
        Self {
            decision_id: Uuid::new_v4(),
            target_id,
            action,
            constraints_applied,
            is_approved,
            generated_at: OffsetDateTime::now_utc(),
        }
    }
}
