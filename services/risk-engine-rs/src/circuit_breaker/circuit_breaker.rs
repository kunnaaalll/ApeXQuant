#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CircuitBreakerState {
    Normal = 0,
    Warning = 1,
    Restricted = 2,
    Critical = 3,
    Frozen = 4,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerTransitionError {
    InvalidTransition {
        from: CircuitBreakerState,
        to: CircuitBreakerState,
    },
}

impl CircuitBreakerState {
    pub fn transition_to(
        &self,
        next_state: CircuitBreakerState,
    ) -> Result<CircuitBreakerState, CircuitBreakerTransitionError> {
        let current_severity = *self as i8;
        let next_severity = next_state as i8;

        // If the state stays the same or worsens, it's always allowed instantly.
        // For example, Normal -> Frozen is allowed if risk spikes.
        if next_severity >= current_severity {
            return Ok(next_state);
        }

        // If the state is improving (recovering), it must proceed sequentially step-by-step.
        // E.g., Frozen (4) -> Critical (3) -> Restricted (2) -> Warning (1) -> Normal (0).
        if current_severity - next_severity == 1 {
            return Ok(next_state);
        }

        // Otherwise, it's an invalid jump skipping recovery steps.
        Err(CircuitBreakerTransitionError::InvalidTransition {
            from: *self,
            to: next_state,
        })
    }
}
