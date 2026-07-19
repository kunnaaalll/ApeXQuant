use super::models::ExpectancyMetrics;
use super::states::ExpectancyState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectancyEvent {
    MetricsUpdated {
        id: Uuid,
        metrics: ExpectancyMetrics,
        timestamp: DateTime<Utc>,
    },
    StateTransitioned {
        id: Uuid,
        from: ExpectancyState,
        to: ExpectancyState,
        timestamp: DateTime<Utc>,
    },
}
