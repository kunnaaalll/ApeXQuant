use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::ExpectancyMetrics;
use super::states::ExpectancyState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectancySnapshot {
    pub metrics: ExpectancyMetrics,
    pub state: ExpectancyState,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
