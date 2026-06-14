use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::TimeframeAssessment;
use super::types::TimeframeType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeframeEvent {
    TimeframeEvaluated {
        event_id: Uuid,
        portfolio_id: Uuid,
        timeframe: TimeframeType,
        assessment: TimeframeAssessment,
        timestamp: DateTime<Utc>,
    },
    TimeframeStateChanged {
        event_id: Uuid,
        portfolio_id: Uuid,
        timeframe: TimeframeType,
        from_state: super::states::TimeframeState,
        to_state: super::states::TimeframeState,
        timestamp: DateTime<Utc>,
    },
}

impl TimeframeEvent {
    pub fn event_id(&self) -> Uuid {
        match self {
            Self::TimeframeEvaluated { event_id, .. } => *event_id,
            Self::TimeframeStateChanged { event_id, .. } => *event_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::TimeframeEvaluated { timestamp, .. } => *timestamp,
            Self::TimeframeStateChanged { timestamp, .. } => *timestamp,
        }
    }
}
