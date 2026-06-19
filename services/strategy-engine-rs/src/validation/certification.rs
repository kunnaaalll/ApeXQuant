#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificationState {
    NotCertified,
    Candidate,
    Certified,
}

#[derive(Debug, Clone)]
pub struct CertificationEngine {
    pub state: CertificationState,
}

impl CertificationEngine {
    pub fn new() -> Self {
        Self {
            state: CertificationState::NotCertified,
        }
    }

    pub fn evaluate(
        &mut self,
        parity_pass: bool,
        determinism_pass: bool,
        replay_pass: bool,
        stress_pass: bool,
        benchmark_pass: bool,
    ) {
        if parity_pass && determinism_pass && replay_pass && stress_pass && benchmark_pass {
            if self.state == CertificationState::Candidate {
                self.state = CertificationState::Certified;
            } else if self.state == CertificationState::NotCertified {
                self.state = CertificationState::Candidate;
            }
        } else {
            self.state = CertificationState::NotCertified;
        }
    }
}

impl Default for CertificationEngine {
    fn default() -> Self {
        Self::new()
    }
}
