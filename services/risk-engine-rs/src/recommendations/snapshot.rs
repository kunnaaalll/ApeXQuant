use super::models::{RecommendationExplanation, RiskRecommendation, TradeAdmissionPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationSnapshot {
    pub recommendation: RiskRecommendation,
    pub admission_policy: TradeAdmissionPolicy,
    pub explanation: RecommendationExplanation,
    pub timestamp: u64,
    pub version: u32,
}

impl RecommendationSnapshot {
    pub fn rebuild(
        recommendation: RiskRecommendation,
        admission_policy: TradeAdmissionPolicy,
        explanation: RecommendationExplanation,
        timestamp: u64,
        version: u32,
    ) -> Self {
        Self {
            recommendation,
            admission_policy,
            explanation,
            timestamp,
            version,
        }
    }
}
