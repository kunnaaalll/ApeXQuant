use crate::shadow::comparison::ComparisonState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorState {
    NotReady,
    Monitoring,
    Candidate,
    Approved,
    Rejected,
}

pub struct GoLiveValidator {
    pub state: ValidatorState,
    pub consecutive_matches: u64,
}

impl Default for GoLiveValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoLiveValidator {
    pub fn new() -> Self {
        Self {
            state: ValidatorState::NotReady,
            consecutive_matches: 0,
        }
    }

    pub fn process(&mut self, comparison: &ComparisonState) {
        if *comparison == ComparisonState::Critical {
            self.state = ValidatorState::Rejected;
            self.consecutive_matches = 0;
            return;
        }

        if *comparison == ComparisonState::ExactMatch || *comparison == ComparisonState::CloseMatch
        {
            self.consecutive_matches += 1;
        } else {
            self.consecutive_matches = 0;
            if self.state == ValidatorState::Approved || self.state == ValidatorState::Candidate {
                self.state = ValidatorState::Monitoring;
            }
        }

        self.evaluate_promotion();
    }

    fn evaluate_promotion(&mut self) {
        match self.state {
            ValidatorState::NotReady | ValidatorState::Rejected => {
                if self.consecutive_matches >= 100 {
                    self.state = ValidatorState::Monitoring;
                }
            }
            ValidatorState::Monitoring => {
                if self.consecutive_matches >= 1000 {
                    self.state = ValidatorState::Candidate;
                }
            }
            ValidatorState::Candidate => {
                if self.consecutive_matches >= 10000 {
                    self.state = ValidatorState::Approved;
                }
            }
            ValidatorState::Approved => {}
        }
    }
}
