use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowState {
    ShadowBooting,
    ShadowWarmup,
    ShadowCollecting,
    ShadowValidating,
    ShadowCandidate,
    ShadowApproved,
    ShadowPaused,
    ShadowFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalShadowOrchestrator {
    pub current_state: ShadowState,
    pub session_id: String,
    pub total_ticks_processed: u64,
}

impl GlobalShadowOrchestrator {
    pub fn new(session_id: String) -> Self {
        Self {
            current_state: ShadowState::ShadowBooting,
            session_id,
            total_ticks_processed: 0,
        }
    }

    pub fn transition_to(&mut self, new_state: ShadowState) -> Result<(), &'static str> {
        // Enforce valid transitions here if needed
        self.current_state = new_state;
        Ok(())
    }

    pub fn tick(&mut self) {
        self.total_ticks_processed += 1;
    }
}
