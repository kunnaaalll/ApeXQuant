pub mod confidence_levels;
pub mod events;
pub mod expected_shortfall;
pub mod historical_var;
pub mod parametric_var;
pub mod snapshot;
pub mod tail_risk;

#[cfg(test)]
mod tests;

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Normal,
    Elevated,
    High,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TailRiskExplanation {
    pub why_risk_increased: String,
    pub triggered_confidence_level: confidence_levels::ConfidenceLevel,
    pub largest_loss_observed: Decimal,
    pub expected_shortfall_contribution: Decimal,
    pub what_prevented_higher_severity: String,
}
