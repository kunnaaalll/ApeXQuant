use crate::validation::state::ValidationState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSnapshot {
    pub state: ValidationState,
}

impl ValidationSnapshot {
    pub const fn new(state: ValidationState) -> Self {
        Self { state }
    }
}
