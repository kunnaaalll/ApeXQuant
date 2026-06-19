use super::adaptive_state::AdaptiveState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveSnapshot {
    pub state: AdaptiveState,
}

impl AdaptiveSnapshot {
    pub fn new(state: AdaptiveState) -> Self {
        Self { state }
    }
}
