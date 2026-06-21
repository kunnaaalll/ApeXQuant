use super::{PolicyError, PolicyState};

pub struct GtcPolicy {
    pub state: PolicyState,
}

impl GtcPolicy {
    pub fn new() -> Self {
        Self {
            state: PolicyState::New,
        }
    }

    pub fn transition(&mut self, to: PolicyState) -> Result<(), PolicyError> {
        let valid = match (self.state, to) {
            (PolicyState::New, PolicyState::Active) => true,
            (PolicyState::New, PolicyState::Rejected) => true,
            (PolicyState::Active, PolicyState::PartiallyFilled) => true,
            (PolicyState::Active, PolicyState::Filled) => true,
            (PolicyState::Active, PolicyState::Cancelled) => true,
            (PolicyState::PartiallyFilled, PolicyState::Filled) => true,
            (PolicyState::PartiallyFilled, PolicyState::PartiallyFilled) => true,
            (PolicyState::PartiallyFilled, PolicyState::Cancelled) => true,
            _ => false,
        };

        if valid {
            self.state = to;
            Ok(())
        } else {
            Err(PolicyError::InvalidTransition {
                policy: "GTC",
                from: self.state,
                to,
            })
        }
    }
}
