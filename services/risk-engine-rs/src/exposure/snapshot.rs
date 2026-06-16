use serde::{Deserialize, Serialize};

use crate::exposure::events::ExposureRiskEvent;
use crate::exposure::exposure_state::ExposureRiskState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureRiskSnapshot {
    pub version: u32,
    pub timestamp: i64,
    pub state: ExposureRiskState,
    pub triggering_event: Option<ExposureRiskEvent>,
}

impl ExposureRiskSnapshot {
    pub fn new(
        version: u32,
        timestamp: i64,
        state: ExposureRiskState,
        triggering_event: Option<ExposureRiskEvent>,
    ) -> Self {
        Self {
            version,
            timestamp,
            state,
            triggering_event,
        }
    }

    pub fn state(&self) -> &ExposureRiskState {
        &self.state
    }
}
