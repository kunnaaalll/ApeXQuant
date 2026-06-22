use super::{PolicyError, PolicyState};

pub struct FokPolicy {
    pub state: PolicyState,
}

impl Default for FokPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl FokPolicy {
    pub fn new() -> Self {
        Self {
            state: PolicyState::New,
        }
    }

    pub fn transition(&mut self, to: PolicyState) -> Result<(), PolicyError> {
        let valid = match (self.state, to) {
            (PolicyState::New, PolicyState::Active) => true,
            (PolicyState::New, PolicyState::Rejected) => true,
            // FOK cannot be partially filled. Must be Filled or Cancelled.
            (PolicyState::Active, PolicyState::Filled) => true,
            (PolicyState::Active, PolicyState::Cancelled) => true,
            _ => false,
        };

        if valid {
            self.state = to;
            Ok(())
        } else {
            Err(PolicyError::InvalidTransition {
                policy: "FOK",
                from: self.state,
                to,
            })
        }
    }
}
