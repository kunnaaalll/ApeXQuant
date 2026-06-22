use super::{PolicyError, PolicyState};

pub struct LimitPolicy {
    pub state: PolicyState,
}

impl Default for LimitPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl LimitPolicy {
    pub fn new() -> Self {
        Self {
            state: PolicyState::New,
        }
    }

    pub fn transition(&mut self, to: PolicyState) -> Result<(), PolicyError> {
        let valid = matches!(
            (self.state, to),
            (PolicyState::New, PolicyState::Active)
                | (PolicyState::New, PolicyState::Rejected)
                | (PolicyState::Active, PolicyState::PartiallyFilled)
                | (PolicyState::Active, PolicyState::Filled)
                | (PolicyState::Active, PolicyState::Cancelled)
                | (PolicyState::PartiallyFilled, PolicyState::Filled)
                | (PolicyState::PartiallyFilled, PolicyState::PartiallyFilled)
                | (PolicyState::PartiallyFilled, PolicyState::Cancelled)
        );

        if valid {
            self.state = to;
            Ok(())
        } else {
            Err(PolicyError::InvalidTransition {
                policy: "Limit",
                from: self.state,
                to,
            })
        }
    }
}
