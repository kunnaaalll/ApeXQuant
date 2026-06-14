use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::SessionAssessment;
use super::types::SessionType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub portfolio_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub assessments: HashMap<SessionType, SessionAssessment>,
    pub version: u64,
}

impl SessionSnapshot {
    pub fn new(portfolio_id: Uuid, timestamp: DateTime<Utc>) -> Self {
        Self {
            portfolio_id,
            timestamp,
            assessments: HashMap::new(),
            version: 1,
        }
    }
}
