//! Risk narrative generator
//!
//! Generates structured, human-readable risk explanations from current and previous
//! RiskInputs. Every intervention explains: Rule, Threshold, Observed Value, Decision,
//! and Recommended Action. No mocked previous state.

use crate::recommendations::models::{RecommendationExplanation, RiskInputs, TradeAdmissionPolicy};

use super::constraints::detect_constraints;
use super::contributors::find_largest_contributor;
use super::deterioration::detect_deterioration;
use super::improvements::detect_improvements;
use super::reasons::track_reasons;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskNarrative {
    pub explanation: RecommendationExplanation,
    /// Machine-readable structured breakdown of the decision
    pub structured: RiskDecisionSummary,
}

/// Structured risk decision — every field is factual, no mocked data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskDecisionSummary {
    pub decision: String,
    pub rule_violations: Vec<RuleViolation>,
    pub recommended_action: String,
}

/// A single rule violation with observed vs threshold values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleViolation {
    pub rule: String,
    pub threshold: String,
    pub observed: String,
    pub blocked: bool,
}

pub fn generate_narrative(
    inputs: &RiskInputs,
    previous_inputs: Option<&RiskInputs>,
    policy: TradeAdmissionPolicy,
    why: String,
) -> RiskNarrative {
    let reasons = track_reasons(
        &format!("{:?}", inputs.drawdown_state),
        &format!("{:?}", inputs.exposure_state),
        &format!("{:?}", inputs.correlation_severity),
        &format!("{:?}", inputs.tail_risk_score),
        &format!("{:?}", inputs.var_severity),
        &format!("{:?}", inputs.circuit_breaker_state),
    );

    let dominant_factor =
        find_largest_contributor(&reasons).unwrap_or_else(|| "None".to_string());

    // Compute improvements and deteriorations from real previous state
    let (improvements, deterioration) = match previous_inputs {
        Some(prev) => {
            let impr = detect_improvements_from_states(prev, inputs);
            let deter = detect_deterioration_from_states(prev, inputs);
            (impr, deter)
        }
        None => (vec![], vec![]),
    };

    let constraint =
        detect_constraints(inputs, policy).unwrap_or_else(|| "No active constraints".to_string());

    let imp_str = if improvements.is_empty() {
        "None".to_string()
    } else {
        improvements.join(", ")
    };

    let det_str = if deterioration.is_empty() {
        "None".to_string()
    } else {
        deterioration.join(", ")
    };

    // Build structured decision summary
    let rule_violations = build_rule_violations(inputs, &reasons);
    let decision = if rule_violations.iter().any(|v| v.blocked) {
        "BLOCKED".to_string()
    } else {
        "APPROVED".to_string()
    };

    let recommended_action = build_recommendation(&rule_violations, &decision);

    RiskNarrative {
        explanation: RecommendationExplanation {
            why,
            what_improved: imp_str,
            what_deteriorated: det_str,
            dominant_factor,
            prevented_stronger_recommendation: constraint,
        },
        structured: RiskDecisionSummary {
            decision,
            rule_violations,
            recommended_action,
        },
    }
}

/// Build structured rule violations from the risk inputs.
fn build_rule_violations(
    inputs: &RiskInputs,
    reasons: &[super::reasons::Reason],
) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    for reason in reasons {
        if reason.severity > 0 {
            let blocked = reason.severity >= 50;
            violations.push(RuleViolation {
                rule: reason.category.clone(),
                threshold: "Acceptable".to_string(),
                observed: reason.description.clone(),
                blocked,
            });
        }
    }

    violations
}

/// Detect improvements by comparing current vs previous state strings.
fn detect_improvements_from_states(prev: &RiskInputs, curr: &RiskInputs) -> Vec<String> {
    let mut improvements = Vec::new();

    // Compare drawdown: if severity went down, it improved
    let prev_dd = severity_rank(&format!("{:?}", prev.drawdown_state));
    let curr_dd = severity_rank(&format!("{:?}", curr.drawdown_state));
    if curr_dd < prev_dd {
        improvements.push(format!("Drawdown improved: {:?} → {:?}", prev.drawdown_state, curr.drawdown_state));
    }

    let prev_exp = severity_rank(&format!("{:?}", prev.exposure_state));
    let curr_exp = severity_rank(&format!("{:?}", curr.exposure_state));
    if curr_exp < prev_exp {
        improvements.push(format!("Exposure improved: {:?} → {:?}", prev.exposure_state, curr.exposure_state));
    }

    let prev_corr = severity_rank(&format!("{:?}", prev.correlation_severity));
    let curr_corr = severity_rank(&format!("{:?}", curr.correlation_severity));
    if curr_corr < prev_corr {
        improvements.push(format!("Correlation risk improved: {:?} → {:?}", prev.correlation_severity, curr.correlation_severity));
    }

    improvements
}

/// Detect deteriorations by comparing current vs previous state strings.
fn detect_deterioration_from_states(prev: &RiskInputs, curr: &RiskInputs) -> Vec<String> {
    let mut deterioration = Vec::new();

    let prev_dd = severity_rank(&format!("{:?}", prev.drawdown_state));
    let curr_dd = severity_rank(&format!("{:?}", curr.drawdown_state));
    if curr_dd > prev_dd {
        deterioration.push(format!("Drawdown worsened: {:?} → {:?}", prev.drawdown_state, curr.drawdown_state));
    }

    let prev_exp = severity_rank(&format!("{:?}", prev.exposure_state));
    let curr_exp = severity_rank(&format!("{:?}", curr.exposure_state));
    if curr_exp > prev_exp {
        deterioration.push(format!("Exposure worsened: {:?} → {:?}", prev.exposure_state, curr.exposure_state));
    }

    let prev_corr = severity_rank(&format!("{:?}", prev.correlation_severity));
    let curr_corr = severity_rank(&format!("{:?}", curr.correlation_severity));
    if curr_corr > prev_corr {
        deterioration.push(format!("Correlation risk worsened: {:?} → {:?}", prev.correlation_severity, curr.correlation_severity));
    }

    deterioration
}

fn severity_rank(desc: &str) -> u32 {
    match desc {
        s if s.contains("Frozen") || s.contains("Collapse") || s.contains("Critical") => 3,
        s if s.contains("High") || s.contains("Warning") || s.contains("Hot") => 2,
        s if s.contains("Healthy") || s.contains("Safe") || s.contains("Normal") => 0,
        _ => 1,
    }
}

fn build_recommendation(violations: &[RuleViolation], decision: &str) -> String {
    if decision == "APPROVED" {
        return "Proceed with trade at approved size".to_string();
    }

    // Find most severe violation
    let blocking: Vec<&RuleViolation> = violations.iter().filter(|v| v.blocked).collect();
    if blocking.is_empty() {
        return "Reduce position size and monitor risk metrics".to_string();
    }

    let primary = &blocking[0];
    format!(
        "Trade blocked by {} ({}). Recommended: wait for {} to normalise before retrying.",
        primary.rule,
        primary.observed,
        primary.rule
    )
}
