use super::models::DegradationAssessment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationSnapshot {
    pub assessment: DegradationAssessment,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
