use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificationState {
    NotReady,
    Experimental,
    ShadowCertified,
    ProductionCertified,
    InstitutionalCertified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationResult {
    pub state: CertificationState,
    pub passes_parity: bool,
    pub passes_determinism: bool,
    pub passes_benchmark: bool,
    pub passes_replay: bool,
    pub passes_monte_carlo: bool,
    pub passes_stress_tests: bool,
}

pub struct CertificationEngine;

impl CertificationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn certify(&self, result: &CertificationResult) -> CertificationState {
        if result.passes_parity && result.passes_determinism && result.passes_benchmark && result.passes_replay && result.passes_monte_carlo && result.passes_stress_tests {
            CertificationState::InstitutionalCertified
        } else if result.passes_parity && result.passes_determinism && result.passes_benchmark {
            CertificationState::ProductionCertified
        } else if result.passes_parity {
            CertificationState::ShadowCertified
        } else if result.passes_determinism {
            CertificationState::Experimental
        } else {
            CertificationState::NotReady
        }
    }
}
