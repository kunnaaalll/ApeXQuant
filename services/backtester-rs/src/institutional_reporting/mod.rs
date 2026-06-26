//! Institutional Reporting Module
//!
//! Generates standardized validation and certification reports.

use crate::certification::CertificationStage;

use crate::production_validator::ProductionValidationReport;

pub trait ReportFormatter {
    fn to_json(&self) -> String;
    fn to_markdown(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct CertificationReport {
    pub strategy_id: String,
    pub previous_stage: CertificationStage,
    pub new_stage: CertificationStage,
    pub approved: bool,
}

impl ReportFormatter for CertificationReport {
    fn to_json(&self) -> String {
        format!(
            r#"{{"strategy_id":"{}","approved":{}}}"#,
            self.strategy_id, self.approved
        )
    }

    fn to_markdown(&self) -> String {
        format!(
            "# Certification Report\n\n**Strategy ID**: {}\n**Approved**: {}",
            self.strategy_id, self.approved
        )
    }
}

#[derive(Debug, Clone)]
pub struct DeploymentRecommendation {
    pub strategy_id: String,
    pub recommendation: String,
    pub production_readiness_score: rust_decimal::Decimal,
}

impl ReportFormatter for DeploymentRecommendation {
    fn to_json(&self) -> String {
        format!(
            r#"{{"strategy_id":"{}","recommendation":"{}","score":"{}"}}"#,
            self.strategy_id, self.recommendation, self.production_readiness_score
        )
    }

    fn to_markdown(&self) -> String {
        format!(
            "# Deployment Recommendation\n\n**Strategy ID**: {}\n**Recommendation**: {}\n**Score**: {}",
            self.strategy_id, self.recommendation, self.production_readiness_score
        )
    }
}

pub struct ReportEngine;

impl ReportEngine {
    pub fn generate_certification_report(
        &self,
        strategy_id: String,
        previous_stage: CertificationStage,
        new_stage: CertificationStage,
        approved: bool,
    ) -> CertificationReport {
        CertificationReport {
            strategy_id,
            previous_stage,
            new_stage,
            approved,
        }
    }

    pub fn generate_deployment_recommendation(
        &self,
        strategy_id: String,
        production_report: &ProductionValidationReport,
    ) -> DeploymentRecommendation {
        let recommendation = if production_report.is_ready {
            "Deploy to Production".to_string()
        } else {
            "Do Not Deploy".to_string()
        };

        DeploymentRecommendation {
            strategy_id,
            recommendation,
            production_readiness_score: production_report.production_readiness_score,
        }
    }
}
