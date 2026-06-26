use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    SuccessfulDecision,
    FailedDecision,
    ModelPerformance,
    RegimeBehavior,
    ResearchOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalMemory {
    pub id: Uuid,
    pub memory_type: MemoryType,
    pub reference_id: Uuid, // ID to the decision, model, or research
    pub score: Decimal,     // Effectiveness or performance score
    pub context: String,    // Regime or context description
    pub recorded_at: OffsetDateTime,
}
