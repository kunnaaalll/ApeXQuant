use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::RegimeAssessment;
use super::types::RegimeType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegimeEvent {
    RegimeEvaluated {
        event_id: Uuid,
        portfolio_id: Uuid,
        regime: RegimeType,
        assessment: RegimeAssessment,
        timestamp: DateTime<Utc>,
    },
    RegimeStateChanged {
        event_id: Uuid,
        portfolio_id: Uuid,
        regime: RegimeType,
        from_state: super::states::RegimeState,
        to_state: super::states::RegimeState,
        timestamp: DateTime<Utc>,
    },
}

impl RegimeEvent {
    pub fn event_id(&self) -> Uuid {
        match self {
            Self::RegimeEvaluated { event_id, .. } => *event_id,
            Self::RegimeStateChanged { event_id, .. } => *event_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::RegimeEvaluated { timestamp, .. } => *timestamp,
            Self::RegimeStateChanged { timestamp, .. } => *timestamp,
        }
    }
}
