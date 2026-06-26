use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfidenceBreakdown {
    pub base_confidence: Decimal,
    pub market_regime_adjustment: Decimal,
    pub historical_accuracy_adjustment: Decimal,
    pub final_confidence: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceReport {
    pub primary_factors: Vec<String>,
    pub historical_references: Vec<Uuid>,
    pub risk_factors: Vec<String>,
    pub input_evidence: Vec<String>, // New in Phase 4
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionExplanation {
    pub decision_id: Uuid,
    pub confidence: ConfidenceBreakdown,
    pub evidence: EvidenceReport,
    pub supporting_metrics: Vec<(String, Decimal)>,
    pub expected_outcome: String,
    pub previous_similar_decisions: Vec<Uuid>,
    // New in Phase 4:
    pub supporting_engines: Vec<String>,
    pub confidence_sources: Vec<String>,
    pub realized_outcome: Option<String>, // Populated post-execution
}

