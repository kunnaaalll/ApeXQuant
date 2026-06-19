use super::models::{RiskCommitteeDecision, RiskRecommendation, TradeAdmissionPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyError {
    IncreaseWithFreeze,
    IncreaseWithBlock,
    AllowWithEmergencyReduction,
}

pub fn validate_consistency(decision: &RiskCommitteeDecision) -> Result<(), ConsistencyError> {
    if decision.recommendation == RiskRecommendation::IncreaseRisk
        && decision.admission_policy == TradeAdmissionPolicy::Freeze
    {
        return Err(ConsistencyError::IncreaseWithFreeze);
    }

    if decision.recommendation == RiskRecommendation::IncreaseRisk
        && decision.admission_policy == TradeAdmissionPolicy::Block
    {
        return Err(ConsistencyError::IncreaseWithBlock);
    }

    if decision.recommendation == RiskRecommendation::EmergencyReduction
        && decision.admission_policy == TradeAdmissionPolicy::Allow
    {
        return Err(ConsistencyError::AllowWithEmergencyReduction);
    }

    Ok(())
}
