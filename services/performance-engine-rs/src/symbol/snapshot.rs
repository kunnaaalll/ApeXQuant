use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::SymbolAssessment;
use super::types::Symbol;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSnapshot {
    pub portfolio_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub assessments: HashMap<Symbol, SymbolAssessment>,
    pub version: u64,
}

impl SymbolSnapshot {
    pub fn new(portfolio_id: Uuid, timestamp: DateTime<Utc>) -> Self {
        Self {
            portfolio_id,
            timestamp,
            assessments: HashMap::new(),
            version: 1,
        }
    }
}
