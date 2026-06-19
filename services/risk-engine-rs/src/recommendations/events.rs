use super::models::{RiskCommitteeDecision, RiskRecommendation};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecommendationEvent {
    RecommendationGenerated(RiskCommitteeDecision),
    RecommendationChanged {
        old: RiskRecommendation,
        new: RiskRecommendation,
        timestamp: u64,
    },
    TradeBlocked {
        reason: String,
        timestamp: u64,
    },
    TradeFrozen {
        reason: String,
        timestamp: u64,
    },
    EmergencyReductionTriggered {
        reason: String,
        timestamp: u64,
    },
}
