use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiEngineEvent {
    Decision(DecisionEvent),
    Recommendation(RecommendationEvent),
    Governance(GovernanceEvent),
    Approval(ApprovalEvent),
    Escalation(EscalationEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionEvent {
    pub decision_id: Uuid,
    pub timestamp: u64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendationEvent {
    pub package_id: Uuid,
    pub timestamp: u64,
    pub recommendation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceEvent {
    pub event_id: Uuid,
    pub timestamp: u64,
    pub action_validated: String,
    pub is_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalEvent {
    pub package_id: Uuid,
    pub timestamp: u64,
    pub approver_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscalationEvent {
    pub escalation_id: Uuid,
    pub timestamp: u64,
    pub reason: String,
}

// Snapshots for quick recovery

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionSnapshot {
    pub latest_decision_id: Uuid,
    pub active_decisions_count: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendationSnapshot {
    pub active_recommendations: Vec<Uuid>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceSnapshot {
    pub total_validations: u64,
    pub total_denials: u64,
    pub timestamp: u64,
}
