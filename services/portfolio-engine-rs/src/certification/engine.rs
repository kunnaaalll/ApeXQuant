use crate::validation::certification::{PortfolioCertificationEngine, CertificationLevel, PortfolioCertification};
use super::state::{CertificationState, CertificationStatus};

pub struct StateTrackerEngine {
    certification_engine: PortfolioCertificationEngine,
}

impl Default for StateTrackerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateTrackerEngine {
    pub fn new() -> Self {
        Self {
            certification_engine: PortfolioCertificationEngine::new(),
        }
    }

    pub fn evaluate_state(&self, mut current_state: CertificationState, latest_certification: &PortfolioCertification) -> CertificationState {
        match latest_certification.level {
            CertificationLevel::Certified => {
                if current_state.status != CertificationStatus::Certified {
                    current_state.status = CertificationStatus::Certified;
                    current_state.certification_date = Some(1234567890); // Placeholder for actual timestamp
                    current_state.revocation_reason = None;
                }
            }
            CertificationLevel::Fail => {
                if current_state.status == CertificationStatus::Certified {
                    current_state.status = CertificationStatus::Revoked;
                    current_state.revocation_reason = Some("Failed certification checks".to_string());
                } else {
                    current_state.status = CertificationStatus::Pending;
                }
            }
            _ => {
                // Warning or Pass (not fully certified)
                if current_state.status == CertificationStatus::Certified {
                    // Decide if warning revokes certification
                    current_state.status = CertificationStatus::Evaluating;
                }
            }
        }
        
        current_state
    }
}
