#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificationState {
    NotCertified,
    Candidate,
    Certified,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct CertificationEngine {
    state: CertificationState,
}

impl Default for CertificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificationEngine {
    pub const fn new() -> Self {
        Self {
            state: CertificationState::NotCertified,
        }
    }

    pub fn process_results(
        &mut self,
        parity_pass: bool,
        determinism_pass: bool,
        replay_pass: bool,
        stress_pass: bool,
        benchmark_pass: bool,
    ) {
        let all_pass = parity_pass && determinism_pass && replay_pass && stress_pass && benchmark_pass;

        if all_pass {
            self.state = match self.state {
                CertificationState::NotCertified => CertificationState::Candidate,
                CertificationState::Candidate => CertificationState::Certified,
                CertificationState::Certified => CertificationState::Certified,
                CertificationState::Rejected => CertificationState::Rejected,
            };
        } else {
            self.state = match self.state {
                CertificationState::Certified => CertificationState::Candidate,
                CertificationState::Candidate => CertificationState::Rejected,
                _ => CertificationState::NotCertified,
            }
        }
    }

    pub fn reset_rejected(&mut self) {
        if self.state == CertificationState::Rejected {
            self.state = CertificationState::NotCertified;
        }
    }

    pub fn current_state(&self) -> CertificationState {
        self.state
    }
}
