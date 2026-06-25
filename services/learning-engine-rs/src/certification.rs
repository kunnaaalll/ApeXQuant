use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationState {
    NotCertified,
    Candidate,
    Certified,
    EliteCertified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationStatus {
    pub strategy_id: String,
    pub state: CertificationState,
    pub history: Vec<CertificationState>,
}

impl CertificationStatus {
    pub fn new(strategy_id: String) -> Self {
        Self {
            strategy_id,
            state: CertificationState::NotCertified,
            history: vec![CertificationState::NotCertified],
        }
    }

    pub fn transition_to(&mut self, new_state: CertificationState) {
        if self.state != new_state {
            self.state = new_state;
            self.history.push(new_state);
        }
    }
}
