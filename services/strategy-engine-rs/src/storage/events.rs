use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub aggregate_id: String,
    pub sequence: i64,
    pub timestamp: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthEvent {
    pub health_score: Decimal,
    pub status: String,
    pub streak: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfidenceEvent {
    pub confidence_score: Decimal,
    pub tier: String,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftEvent {
    pub drift_score: Decimal,
    pub requires_retraining: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationEvent {
    pub recommended_allocation: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendationEvent {
    pub action: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradationEvent {
    pub is_degraded: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaEvent {
    pub learning_rate: Decimal,
    pub adaptation_score: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterEvent {
    pub cluster_id: String,
    pub similarity_score: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextEvent {
    pub market_regime: String,
    pub volatility_index: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvent {
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowEvent {
    pub shadow_pnl: Decimal,
    pub is_diverging: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StrategyEventWrapper {
    Health(HealthEvent),
    Confidence(ConfidenceEvent),
    Drift(DriftEvent),
    Allocation(AllocationEvent),
    Recommendation(RecommendationEvent),
    Degradation(DegradationEvent),
    Meta(MetaEvent),
    Cluster(ClusterEvent),
    Context(ContextEvent),
    Validation(ValidationEvent),
    Shadow(ShadowEvent),
}
