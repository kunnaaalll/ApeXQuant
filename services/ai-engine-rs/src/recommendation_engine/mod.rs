use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationType {
    AllocationRecommendation { strategy_id: Uuid, amount: Decimal },
    StrategyPromotion { strategy_id: Uuid },
    StrategyRetirement { strategy_id: Uuid, reason: String },
    ExposureReduction { strategy_id: Uuid, reduction_percentage: Decimal },
    RiskIntervention { strategy_id: Uuid, risk_type: String },
    ResearchLaunch { topic: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplanationBundle {
    pub confidence_breakdown_id: Uuid,
    pub historical_references: Vec<Uuid>,
    pub similar_decisions: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Recommendation {
    pub recommendation_id: Uuid,
    pub recommendation_type: RecommendationType,
    pub confidence_score: Decimal, // 0-100
    pub supporting_evidence: Vec<String>,
    pub expected_impact: String,
    pub risk_assessment: String,
    pub explanation_bundle: ExplanationBundle,
    pub generated_at: OffsetDateTime,
}

impl Recommendation {
    pub fn new(
        recommendation_type: RecommendationType,
        confidence_score: Decimal,
        supporting_evidence: Vec<String>,
        expected_impact: String,
        risk_assessment: String,
        explanation_bundle: ExplanationBundle,
    ) -> Result<Self, &'static str> {
        if confidence_score < Decimal::ZERO || confidence_score > Decimal::ONE_HUNDRED {
            return Err("Confidence score must be between 0 and 100");
        }

        Ok(Self {
            recommendation_id: Uuid::new_v4(),
            recommendation_type,
            confidence_score,
            supporting_evidence,
            expected_impact,
            risk_assessment,
            explanation_bundle,
            generated_at: OffsetDateTime::now_utc(),
        })
    }
}
