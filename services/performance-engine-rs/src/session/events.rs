use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::SessionAssessment;
use super::types::SessionType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionEvent {
    SessionEvaluated {
        event_id: Uuid,
        portfolio_id: Uuid,
        session: SessionType,
        assessment: SessionAssessment,
        timestamp: DateTime<Utc>,
    },
    SessionStateChanged {
        event_id: Uuid,
        portfolio_id: Uuid,
        session: SessionType,
        from_state: super::states::SessionState,
        to_state: super::states::SessionState,
        timestamp: DateTime<Utc>,
    },
}

impl SessionEvent {
    pub fn event_id(&self) -> Uuid {
        match self {
            Self::SessionEvaluated { event_id, .. } => *event_id,
            Self::SessionStateChanged { event_id, .. } => *event_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::SessionEvaluated { timestamp, .. } => *timestamp,
            Self::SessionStateChanged { timestamp, .. } => *timestamp,
        }
    }
}
