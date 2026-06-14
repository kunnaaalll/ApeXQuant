use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::models::StabilityMetrics;
use super::states::StabilityState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilityEvent {
    MetricsUpdated {
        id: Uuid,
        metrics: StabilityMetrics,
        timestamp: DateTime<Utc>,
    },
    StateTransitioned {
        id: Uuid,
        from: StabilityState,
        to: StabilityState,
        timestamp: DateTime<Utc>,
    },
}
