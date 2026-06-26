use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub score: Decimal,
    pub threshold: Decimal,
    pub is_confident: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationReport {
    pub report_id: Uuid,
    pub inference_id: Uuid,
    pub summary: String,
    pub details: Vec<String>,
    pub generated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub inference_id: Uuid,
    pub model_id: Uuid,
    pub predicted_value: Decimal,
    pub confidence: ConfidenceScore,
    pub explanation: ExplanationReport,
    pub computed_at: OffsetDateTime,
}
