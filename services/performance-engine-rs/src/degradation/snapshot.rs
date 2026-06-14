use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::DegradationAssessment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationSnapshot {
    pub assessment: DegradationAssessment,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
