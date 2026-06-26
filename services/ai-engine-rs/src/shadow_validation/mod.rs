use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowComparison {
    pub decision_id: Uuid,
    pub ai_recommendation_id: Uuid,
    pub human_decision_id: Option<Uuid>,
    pub actual_outcome_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowMetrics {
    pub comparison_id: Uuid,
    pub agreement_score: Decimal, // 0-100
    pub confidence_calibration: Decimal, // 0-100
    pub recommendation_drift: Decimal, // 0-100
    pub recommendation_quality: Decimal, // 0-100
}
