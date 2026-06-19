use super::allocation_state::AllocationState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocationSnapshot {
    pub state: AllocationState,
}

impl AllocationSnapshot {
    pub fn new(state: AllocationState) -> Self {
        Self { state }
    }
}
