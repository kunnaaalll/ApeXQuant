use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::models::BaseAnalytics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsEvent {
    Updated {
        id: Uuid,
        analytics: BaseAnalytics,
        timestamp: DateTime<Utc>,
    },
}
