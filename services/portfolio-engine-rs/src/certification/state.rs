use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationStatus {
    Pending,
    Evaluating,
    Certified,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationState {
    pub account_id: String,
    pub status: CertificationStatus,
    pub certification_date: Option<u64>,
    pub revocation_reason: Option<String>,
}

impl Default for CertificationState {
    fn default() -> Self {
        Self {
            account_id: String::new(),
            status: CertificationStatus::Pending,
            certification_date: None,
            revocation_reason: None,
        }
    }
}
