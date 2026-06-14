use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{events::PortfolioEvent, state::PortfolioState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SnapshotFrequency {
    Realtime, // Every event
    M1,
    M5,
    M15,
    H1,
    D1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioSnapshot {
    pub id: Uuid,
    pub version: u64,
    pub timestamp: OffsetDateTime,
    pub state: PortfolioState,
    pub trigger_event: PortfolioEvent,
}

impl PortfolioSnapshot {
    pub fn new(
        version: u64,
        state: PortfolioState,
        trigger_event: PortfolioEvent,
        timestamp: OffsetDateTime,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            version,
            timestamp,
            state,
            trigger_event,
        }
    }
}
