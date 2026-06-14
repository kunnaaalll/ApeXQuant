use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::SymbolAssessment;
use super::types::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolEvent {
    SymbolEvaluated {
        event_id: Uuid,
        portfolio_id: Uuid,
        symbol: Symbol,
        assessment: SymbolAssessment,
        timestamp: DateTime<Utc>,
    },
    SymbolStateChanged {
        event_id: Uuid,
        portfolio_id: Uuid,
        symbol: Symbol,
        from_state: super::states::SymbolState,
        to_state: super::states::SymbolState,
        timestamp: DateTime<Utc>,
    },
}

impl SymbolEvent {
    pub fn event_id(&self) -> Uuid {
        match self {
            Self::SymbolEvaluated { event_id, .. } => *event_id,
            Self::SymbolStateChanged { event_id, .. } => *event_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::SymbolEvaluated { timestamp, .. } => *timestamp,
            Self::SymbolStateChanged { timestamp, .. } => *timestamp,
        }
    }
}
