use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::RegimeAssessment;
use super::types::RegimeType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegimeSnapshot {
    pub portfolio_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub assessments: HashMap<RegimeType, RegimeAssessment>,
    pub version: u64,
}

impl RegimeSnapshot {
    pub fn new(portfolio_id: Uuid, timestamp: DateTime<Utc>) -> Self {
        Self {
            portfolio_id,
            timestamp,
            assessments: HashMap::new(),
            version: 1,
        }
    }
}
