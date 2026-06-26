use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftType {
    Strategy,
    Execution,
    Market,
    Portfolio,
    Risk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftState {
    Healthy,
    Warning,
    Critical,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftDirection {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendedAction {
    Monitor,
    ReduceExposure,
    HaltTrading,
    RecalibrateModels,
    InvestigateExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriftReport {
    pub drift_id: Uuid,
    pub target_id: Uuid,
    pub drift_type: DriftType,
    pub state: DriftState,
    pub severity: Decimal, // 0 to 100
    pub direction: DriftDirection,
    pub confidence_score: Decimal, // 0 to 100
    pub recommended_action: RecommendedAction,
    pub detected_at: OffsetDateTime,
}

impl DriftReport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        drift_id: Uuid,
        target_id: Uuid,
        drift_type: DriftType,
        state: DriftState,
        severity: Decimal,
        direction: DriftDirection,
        confidence_score: Decimal,
        recommended_action: RecommendedAction,
        detected_at: OffsetDateTime,
    ) -> Result<Self, &'static str> {
        if severity < Decimal::ZERO || severity > Decimal::ONE_HUNDRED {
            return Err("Severity must be between 0 and 100");
        }
        if confidence_score < Decimal::ZERO || confidence_score > Decimal::ONE_HUNDRED {
            return Err("Confidence score must be between 0 and 100");
        }

        Ok(Self {
            drift_id,
            target_id,
            drift_type,
            state,
            severity,
            direction,
            confidence_score,
            recommended_action,
            detected_at,
        })
    }
}
