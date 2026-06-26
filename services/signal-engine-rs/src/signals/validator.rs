use crate::signals::SignalResult;

pub struct SignalValidator {}

impl SignalValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate(&self, _signal: &SignalResult) -> Result<(), String> {
        Ok(())
    }
}
