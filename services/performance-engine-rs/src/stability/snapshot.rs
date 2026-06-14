use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::StabilityMetrics;
use super::states::StabilityState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilitySnapshot {
    pub metrics: StabilityMetrics,
    pub state: StabilityState,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
