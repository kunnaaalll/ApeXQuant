use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismState {
    Deterministic,
    Diverged,
}

#[derive(Debug, Clone)]
pub struct DeterminismValidator;

impl Default for DeterminismValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterminismValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn execute<F>(&self, mut operation: F, initial_state: Decimal) -> DeterminismState
    where
        F: FnMut(Decimal) -> Decimal,
    {
        let mut current_state = initial_state;

        let reference_final_state = {
            let mut state = initial_state;
            for _ in 0..100_000 {
                state = operation(state);
            }
            state
        };

        for _ in 0..100_000 {
            current_state = operation(current_state);
        }

        if current_state == reference_final_state {
            DeterminismState::Deterministic
        } else {
            DeterminismState::Diverged
        }
    }
}
