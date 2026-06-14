use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::EdgeAssessment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub assessment: EdgeAssessment,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
