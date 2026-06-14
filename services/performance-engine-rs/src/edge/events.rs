use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::models::EdgeAssessment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeEvent {
    Assessed {
        id: Uuid,
        assessment: EdgeAssessment,
        timestamp: DateTime<Utc>,
    },
}
