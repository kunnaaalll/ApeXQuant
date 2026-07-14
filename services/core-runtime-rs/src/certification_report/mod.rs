use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationStatus {
    Passed,
    Failed,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationReport {
    pub session_id: String,
    pub replay_hash: String,
    pub parity_score: f64,
    pub drift_score: f64,
    pub pnl: f64,
    pub max_drawdown: f64,
    pub certification_status: CertificationStatus,
}

pub struct CertificationReportGenerator;

impl CertificationReportGenerator {
    pub fn generate_report(
        session_id: String,
        replay_hash: String,
        parity_score: f64,
        drift_score: f64,
        pnl: f64,
        max_drawdown: f64,
        status: CertificationStatus,
    ) -> Result<String, &'static str> {
        let report = CertificationReport {
            session_id,
            replay_hash,
            parity_score,
            drift_score,
            pnl,
            max_drawdown,
            certification_status: status,
        };

        serde_json::to_string_pretty(&report).map_err(|_| "Failed to serialize certification report")
    }
}
