use super::{PolicyError, PolicyState};

pub struct IocPolicy {
    pub state: PolicyState,
}

impl Default for IocPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl IocPolicy {
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
            // IOC partially filled must immediately cancel remaining
            (PolicyState::PartiallyFilled, PolicyState::Cancelled) => true,
            _ => false,
        };

        if valid {
            self.state = to;
            Ok(())
        } else {
            Err(PolicyError::InvalidTransition {
                policy: "IOC",
                from: self.state,
                to,
            })
        }
    }
}
