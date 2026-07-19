use super::models::EdgeAssessment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub assessment: EdgeAssessment,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
