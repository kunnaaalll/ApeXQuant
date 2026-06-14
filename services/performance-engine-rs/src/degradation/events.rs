use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::models::DegradationAssessment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradationEvent {
    Assessed {
        id: Uuid,
        assessment: DegradationAssessment,
        timestamp: DateTime<Utc>,
    },
}
