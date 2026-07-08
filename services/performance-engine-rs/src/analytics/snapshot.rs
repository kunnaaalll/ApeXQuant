use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::StrategyAnalyticsResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub analytics: StrategyAnalyticsResult,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
