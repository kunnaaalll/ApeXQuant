use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::events::ExposureEvent;
use super::state::ExposureState;

// Note: Reusing the SnapshotFrequency enum from portfolio if possible,
// or redefining it for exposure. For modularity, let's redefine it or import it.
// We'll redefine it here to keep exposure decoupled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExposureSnapshotFrequency {
    Realtime,
    M1,
    M5,
    M15,
    H1,
    D1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExposureSnapshot {
    pub id: Uuid,
    pub version: u64,
    pub timestamp: OffsetDateTime,
    pub state: ExposureState,
    pub trigger_event: ExposureEvent,
}

impl ExposureSnapshot {
    pub fn new(
        version: u64,
        state: ExposureState,
        trigger_event: ExposureEvent,
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
