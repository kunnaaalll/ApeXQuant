use super::models::StrategyAnalyticsResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsEvent {
    Updated {
        id: Uuid,
        analytics: StrategyAnalyticsResult,
        timestamp: DateTime<Utc>,
    },
}
