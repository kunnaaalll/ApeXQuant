use crate::validation::state::ValidationState;

pub struct ValidationReporter;

impl ValidationReporter {
    pub fn generate_markdown(state: &ValidationState) -> String {
        let status = match state.certification_state {
            crate::validation::certification::CertificationState::NotCertified => "Not Certified",
            crate::validation::certification::CertificationState::Candidate => "Candidate",
            crate::validation::certification::CertificationState::Certified => "Certified",
            crate::validation::certification::CertificationState::Rejected => "Rejected",
        };
        let health = match state.health {
            crate::validation::health::ValidationHealth::Excellent => "Excellent",
            crate::validation::health::ValidationHealth::Good => "Good",
            crate::validation::health::ValidationHealth::Normal => "Normal",
            crate::validation::health::ValidationHealth::Weak => "Weak",
            crate::validation::health::ValidationHealth::Critical => "Critical",
        };
        
        format!(
            "# Validation Report\n\n\
            ## Certification Status\n- {}\n\
            ## Score\n- {}\n\
            ## Health\n- {}\n\
            ## Parity Score\n- {}\n",
            status,
            state.score.value,
            health,
            state.parity_score
        )
    }

    pub fn generate_json(state: &ValidationState) -> String {
        let status = match state.certification_state {
            crate::validation::certification::CertificationState::NotCertified => "NotCertified",
            crate::validation::certification::CertificationState::Candidate => "Candidate",
            crate::validation::certification::CertificationState::Certified => "Certified",
            crate::validation::certification::CertificationState::Rejected => "Rejected",
        };
        let health = match state.health {
            crate::validation::health::ValidationHealth::Excellent => "Excellent",
            crate::validation::health::ValidationHealth::Good => "Good",
            crate::validation::health::ValidationHealth::Normal => "Normal",
            crate::validation::health::ValidationHealth::Weak => "Weak",
            crate::validation::health::ValidationHealth::Critical => "Critical",
        };

        format!(
            "{{\n\
                \"certification_status\": \"{}\",\n\
                \"score\": \"{}\",\n\
                \"health\": \"{}\",\n\
                \"parity_score\": \"{}\"\n\
            }}",
            status, state.score.value, health, state.parity_score
        )
    }
}
