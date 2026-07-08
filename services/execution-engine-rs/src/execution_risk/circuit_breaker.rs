use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExecutionProtectionState {
    Normal,
    Warning,
    Restricted,
    Critical,
    Frozen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    IllegalTransition,
}

impl ExecutionProtectionState {
    pub fn transition(
        &mut self,
        next_state: ExecutionProtectionState,
    ) -> Result<(), ExecutionError> {
        // Immediate escalation allowed.
        if next_state >= *self {
            *self = next_state;
            return Ok(());
        }

        // Recovery must be sequential.
        let valid_recovery = match self {
            ExecutionProtectionState::Frozen => next_state == ExecutionProtectionState::Critical,
            ExecutionProtectionState::Critical => {
                next_state == ExecutionProtectionState::Restricted
            }
            ExecutionProtectionState::Restricted => next_state == ExecutionProtectionState::Warning,
            ExecutionProtectionState::Warning => next_state == ExecutionProtectionState::Normal,
            ExecutionProtectionState::Normal => true,
        };

        if valid_recovery {
            *self = next_state;
            Ok(())
        } else {
            Err(ExecutionError::IllegalTransition)
        }
    }
}
