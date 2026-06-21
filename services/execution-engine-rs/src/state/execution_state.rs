use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionState {
    Idle,
    Submitting,
    Waiting,
    Filled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StateTransitionError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ExecutionState,
        to: ExecutionState,
    },
}

impl ExecutionState {
    pub fn transition_to(&mut self, new_state: ExecutionState) -> Result<(), StateTransitionError> {
        match (*self, new_state) {
            (ExecutionState::Filled, ExecutionState::Submitting) => {
                Err(StateTransitionError::InvalidTransition {
                    from: *self,
                    to: new_state,
                })
            }
            (ExecutionState::Failed, ExecutionState::Filled) => {
                Err(StateTransitionError::InvalidTransition {
                    from: *self,
                    to: new_state,
                })
            }
            _ => {
                *self = new_state;
                Ok(())
            }
        }
    }
}
