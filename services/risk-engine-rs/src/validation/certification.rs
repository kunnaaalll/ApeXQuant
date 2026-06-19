use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationState {
    NotCertified,
    Candidate,
    Certified,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationResult {
    pub state: CertificationState,
    pub reasons: Vec<String>,
}

pub struct RiskCertificationEngine;

impl Default for RiskCertificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskCertificationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates if the risk engine passes certification.
    /// Requirements:
    /// - Agreement > 99%
    /// - Panics: 0
    /// - Corruption: 0
    /// - Determinism: 100%
    /// - Replay consistency: 100%
    /// - Benchmark targets met: true
    #[allow(clippy::too_many_arguments)]
    pub fn certify(
        &self,
        current_state: CertificationState,
        agreement: Decimal,
        panics: u64,
        corruption: bool,
        determinism: bool,
        replay: bool,
        benchmark_met: bool,
    ) -> Result<CertificationResult, crate::error::RiskError> {
        if current_state == CertificationState::Rejected {
            // Cannot transition from Rejected directly to Certified.
            // Would need to reset to NotCertified first.
            return Ok(CertificationResult {
                state: CertificationState::Rejected,
                reasons: vec!["Cannot transition from Rejected to Certified directly".to_string()],
            });
        }

        let mut reasons = Vec::new();
        let target_agreement = Decimal::new(99, 0);

        if agreement <= target_agreement {
            reasons.push(format!("Agreement {} is not > 99%", agreement));
        }
        if panics > 0 {
            reasons.push(format!("Panics detected: {}", panics));
        }
        if corruption {
            reasons.push("State corruption detected".to_string());
        }
        if !determinism {
            reasons.push("Determinism failed".to_string());
        }
        if !replay {
            reasons.push("Replay consistency failed".to_string());
        }
        if !benchmark_met {
            reasons.push("Benchmark targets not met".to_string());
        }

        let next_state = if reasons.is_empty() {
            match current_state {
                CertificationState::NotCertified => CertificationState::Candidate,
                CertificationState::Candidate => CertificationState::Certified,
                CertificationState::Certified => CertificationState::Certified,
                CertificationState::Rejected => CertificationState::Rejected, // Handled above
            }
        } else {
            CertificationState::Rejected
        };

        Ok(CertificationResult {
            state: next_state,
            reasons,
        })
    }
}
