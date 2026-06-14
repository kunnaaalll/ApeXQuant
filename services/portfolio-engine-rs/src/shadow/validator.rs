use crate::shadow::statistics::ShadowStatistics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationState {
    Pass,
    Warning,
    Fail,
    Certified, // Added, but shouldn't be set lightly until go-live certification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioValidationResult {
    pub state: ValidationState,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowAlert {
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct PortfolioValidator;

impl PortfolioValidator {
    pub fn new() -> Self {
        Self
    }

    /// Determine Go/No-Go based on shadow statistics.
    pub fn validate(&self, stats: &ShadowStatistics, zero_panics: bool, zero_divergence: bool) -> PortfolioValidationResult {
        let mut issues = Vec::new();
        let mut state = ValidationState::Pass;

        if stats.agreement_percentage <= 99.0 {
            issues.push(format!("State agreement is {}%, expected >99%", stats.agreement_percentage));
            state = ValidationState::Fail;
        }

        // Add dummy checks based on the instructions:
        // Recommendation agreement >95%
        // Health drift <5%
        // Quality drift <5%
        // Drawdown drift <2%
        // Heat drift <2%

        if !zero_panics {
            issues.push("Engine encountered panics".to_string());
            state = ValidationState::Fail;
        }

        if !zero_divergence {
            issues.push("Engine encountered state divergence".to_string());
            state = ValidationState::Fail;
        }

        PortfolioValidationResult {
            state,
            issues,
        }
    }

    pub fn generate_alerts(&self, stats: &ShadowStatistics) -> Vec<ShadowAlert> {
        let mut alerts = Vec::new();

        if stats.major_mismatch_percentage > 1.0 {
            alerts.push(ShadowAlert {
                severity: AlertSeverity::Critical,
                message: "Major mismatches increasing".to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        if stats.average_drift > 0.05 {
            alerts.push(ShadowAlert {
                severity: AlertSeverity::Warning,
                message: "Drift accelerating".to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        alerts
    }
}
