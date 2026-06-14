use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::TimeframeAssessment;
use super::types::TimeframeType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeframeSnapshot {
    pub portfolio_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub assessments: HashMap<TimeframeType, TimeframeAssessment>,
    pub version: u64,
}

impl TimeframeSnapshot {
    pub fn new(portfolio_id: Uuid, timestamp: DateTime<Utc>) -> Self {
        Self {
            portfolio_id,
            timestamp,
            assessments: HashMap::new(),
            version: 1,
        }
    }
}
