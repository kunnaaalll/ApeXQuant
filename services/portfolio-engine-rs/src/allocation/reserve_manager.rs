use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::errors::AllocationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapitalReserve {
    pub reserved_amount: Decimal,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityReserveAssessment {
    pub is_exceptional_opportunity: bool,
    pub required_reserve: Decimal,
    pub confidence: Decimal,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveManager {
    pub normal_reserve: Decimal,
    pub emergency_reserve: Decimal,
    pub recovery_reserve: Decimal,
    pub opportunity_reserve: Decimal,
}

impl ReserveManager {
    pub fn new(
        normal_reserve: Decimal,
        emergency_reserve: Decimal,
    ) -> Result<Self, AllocationError> {
        if normal_reserve < Decimal::ZERO {
            return Err(AllocationError::NegativeReserve("Normal reserve cannot be negative".into()));
        }
        if emergency_reserve < Decimal::ZERO {
            return Err(AllocationError::NegativeReserve("Emergency reserve cannot be negative".into()));
        }

        Ok(Self {
            normal_reserve,
            emergency_reserve,
            recovery_reserve: Decimal::ZERO,
            opportunity_reserve: Decimal::ZERO,
        })
    }

    pub fn total_reserved(&self) -> Decimal {
        self.normal_reserve + self.emergency_reserve + self.recovery_reserve + self.opportunity_reserve
    }

    pub fn update_recovery_reserve(&mut self, amount: Decimal) -> Result<(), AllocationError> {
        if amount < Decimal::ZERO {
            return Err(AllocationError::NegativeReserve("Recovery reserve cannot be negative".into()));
        }
        self.recovery_reserve = amount;
        Ok(())
    }

    pub fn update_opportunity_reserve(&mut self, assessment: OpportunityReserveAssessment) -> Result<(), AllocationError> {
        if assessment.required_reserve < Decimal::ZERO {
            return Err(AllocationError::NegativeReserve("Opportunity reserve cannot be negative".into()));
        }
        self.opportunity_reserve = assessment.required_reserve;
        Ok(())
    }

    pub fn can_deploy(&self, amount: Decimal, total_available: Decimal) -> bool {
        // We can never deploy more than total available minus reserves.
        total_available >= (self.total_reserved() + amount)
    }
}
