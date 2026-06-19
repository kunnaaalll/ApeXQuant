use crate::shadow::comparison::ShadowComparisonState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoLiveState {
    NotReady,
    Monitoring,
    Candidate,
    Approved,
}

#[derive(Debug, Clone)]
pub struct GoLiveValidator {
    pub state: GoLiveState,
    pub consecutive_exact_matches: u64,
}

impl Default for GoLiveValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoLiveValidator {
    pub fn new() -> Self {
        Self {
            state: GoLiveState::NotReady,
            consecutive_exact_matches: 0,
        }
    }

    pub fn process(&mut self, comparison: ShadowComparisonState) {
        if comparison == ShadowComparisonState::ExactMatch {
            self.consecutive_exact_matches += 1;
            
            self.state = match self.consecutive_exact_matches {
                0..=99 => GoLiveState::NotReady,
                100..=999 => GoLiveState::Monitoring,
                1000..=9999 => GoLiveState::Candidate,
                _ => GoLiveState::Approved,
            };
        } else {
            // Demotions occur immediately
            self.consecutive_exact_matches = 0;
            self.state = GoLiveState::NotReady;
        }
    }
}
