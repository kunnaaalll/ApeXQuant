#![deny(unsafe_code)]

pub mod clusters;
pub mod correlation_matrix;
pub mod events;
pub mod hidden_leverage;
pub mod snapshot;
pub mod windows;

#[cfg(test)]
mod tests;

pub use clusters::{CorrelationCategory, CorrelationCluster};
pub use correlation_matrix::CorrelationMatrix;
pub use events::CorrelationRiskEvent;
pub use hidden_leverage::{HiddenLeverageAssessment, HiddenLeverageState};
pub use snapshot::CorrelationRiskSnapshot;
pub use windows::{CorrelationWindow, CorrelationWindowType};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CorrelationSeverity {
    Low,
    Moderate,
    Elevated,
    High,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationExplanation {
    pub risk_increased_reason: String,
    pub primary_cluster_cause: Option<CorrelationCategory>,
    pub largest_overlap_dimension: String,
    pub largest_hidden_leverage_contributor: String,
    pub mitigation_reason: Option<String>, // Why stronger severity was prevented
}

impl CorrelationExplanation {
    pub fn new(
        risk_increased_reason: impl Into<String>,
        primary_cluster_cause: Option<CorrelationCategory>,
        largest_overlap_dimension: impl Into<String>,
        largest_hidden_leverage_contributor: impl Into<String>,
        mitigation_reason: Option<impl Into<String>>,
    ) -> Self {
        Self {
            risk_increased_reason: risk_increased_reason.into(),
            primary_cluster_cause,
            largest_overlap_dimension: largest_overlap_dimension.into(),
            largest_hidden_leverage_contributor: largest_hidden_leverage_contributor.into(),
            mitigation_reason: mitigation_reason.map(Into::into),
        }
    }
}
