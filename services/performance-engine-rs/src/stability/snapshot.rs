use super::models::StabilityMetrics;
use super::states::StabilityState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilitySnapshot {
    pub metrics: StabilityMetrics,
    pub state: StabilityState,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
