use super::state::{CertificationState, CertificationStatus};
use crate::validation::certification::{CertificationLevel, PortfolioCertification};

pub struct StateTrackerEngine;

impl Default for StateTrackerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateTrackerEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_state(
        &self,
        mut current_state: CertificationState,
        latest_certification: &PortfolioCertification,
    ) -> CertificationState {
        match latest_certification.level {
            CertificationLevel::Certified => {
                if current_state.status != CertificationStatus::Certified {
                    current_state.status = CertificationStatus::Certified;
                    current_state.certification_date = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    );
                    current_state.revocation_reason = None;
                }
            }
            CertificationLevel::Fail => {
                if current_state.status == CertificationStatus::Certified {
                    current_state.status = CertificationStatus::Revoked;
                    current_state.revocation_reason =
                        Some("Failed certification checks".to_string());
                } else {
                    current_state.status = CertificationStatus::Pending;
                }
            }
            _ => {
                if current_state.status == CertificationStatus::Certified {
                    current_state.status = CertificationStatus::Evaluating;
                }
            }
        }
        current_state
    }

    pub fn generate_audit_artifact(
        &self,
        certification: &PortfolioCertification,
    ) -> std::io::Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let filename = format!("apex_certification_audit_{}.json", timestamp);

        let artifact = format!(
            "{{\n  \"level\": \"{:?}\",\n  \"parity_match\": {},\n  \"replay_exact\": {},\n  \"timestamp\": {}\n}}",
            certification.level,
            certification.parity_result.state_agreement_pct,
            certification.replay_result.exact_match,
            timestamp
        );

        std::fs::write(&filename, artifact)?;
        tracing::info!("Institutional Audit Artifact generated at: {}", filename);

        Ok(())
    }
}
