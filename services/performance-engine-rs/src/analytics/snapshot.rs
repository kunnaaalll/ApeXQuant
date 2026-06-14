use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::BaseAnalytics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub analytics: BaseAnalytics,
    pub last_updated: DateTime<Utc>,
    pub version: u64,
}
