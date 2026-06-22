#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorState {
    NotReady,
    Monitoring,
    Candidate,
    Approved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoLiveValidator {
    pub state: ValidatorState,
    pub consecutive_parity_streaks: u64,
}

impl Default for GoLiveValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoLiveValidator {
    pub const fn new() -> Self {
        Self {
            state: ValidatorState::NotReady,
            consecutive_parity_streaks: 0,
        }
    }

    pub fn process_parity_pass(&mut self) {
        self.consecutive_parity_streaks = self.consecutive_parity_streaks.saturating_add(1);

        if self.consecutive_parity_streaks >= 10 && self.state == ValidatorState::NotReady {
            self.state = ValidatorState::Monitoring;
        } else if self.consecutive_parity_streaks >= 50 && self.state == ValidatorState::Monitoring {
            self.state = ValidatorState::Candidate;
        } else if self.consecutive_parity_streaks >= 100 && self.state == ValidatorState::Candidate {
            self.state = ValidatorState::Approved;
        }
    }

    pub fn process_parity_failure(&mut self) {
        self.consecutive_parity_streaks = 0;

        self.state = match self.state {
            ValidatorState::Approved => ValidatorState::Candidate,
            ValidatorState::Candidate => ValidatorState::Monitoring,
            ValidatorState::Monitoring | ValidatorState::NotReady => ValidatorState::NotReady,
        };
    }
}
