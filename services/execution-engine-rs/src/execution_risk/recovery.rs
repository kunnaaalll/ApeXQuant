use super::circuit_breaker::ExecutionProtectionState;
use super::cooldown::CooldownEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    Locked,
    Recovering,
    Stable,
}

pub struct RecoveryEngine {
    pub current_state: RecoveryState,
    pub cooldown: CooldownEngine,
}

impl RecoveryEngine {
    pub fn new(cooldown: CooldownEngine) -> Self {
        Self {
            current_state: RecoveryState::Stable,
            cooldown,
        }
    }

    pub fn record_failure(&mut self) {
        self.current_state = RecoveryState::Locked;
        self.cooldown.reset();
    }

    pub fn attempt_recovery(
        &mut self,
        protection_state: &mut ExecutionProtectionState,
    ) -> Result<(), super::circuit_breaker::ExecutionError> {
        if self.current_state == RecoveryState::Locked {
            if self.cooldown.is_ready_for_recovery() {
                self.current_state = RecoveryState::Recovering;
            } else {
                return Ok(()); // Still locked, waiting for cooldown
            }
        }

        if self.current_state == RecoveryState::Recovering {
            // Slow sequential recovery
            let next_state = match protection_state {
                ExecutionProtectionState::Frozen => ExecutionProtectionState::Critical,
                ExecutionProtectionState::Critical => ExecutionProtectionState::Restricted,
                ExecutionProtectionState::Restricted => ExecutionProtectionState::Warning,
                ExecutionProtectionState::Warning => ExecutionProtectionState::Normal,
                ExecutionProtectionState::Normal => {
                    self.current_state = RecoveryState::Stable;
                    ExecutionProtectionState::Normal
                }
            };

            if next_state != *protection_state {
                protection_state.transition(next_state)?;
            }
        }

        Ok(())
    }
}
