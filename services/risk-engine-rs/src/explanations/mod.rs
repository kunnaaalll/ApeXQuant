//! Explainability framework for risk decisions
use rust_decimal::prelude::FromPrimitive;

use crate::{
    volatility::VolatilityMetrics, DailyLimitState, DrawdownState, ExposureMetrics,
    PositionSizeResult, StreakState,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// mod builder;
// mod formatter;
// mod tree;

// pub use builder::ExplanationBuilder;
// pub use formatter::ExplanationFormatter;
// pub use tree::{ExplanationNode, ExplanationTree};

/// Complete risk explanation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskExplanation {
    /// Summary of decision
    pub summary: String,
    /// Detailed explanations by category
    pub explanations: Vec<ExplanationEntry>,
    /// Contributing factors
    pub contributing_factors: Vec<Factor>,
    /// Reducing factors
    pub reducing_factors: Vec<Factor>,
    /// Confidence breakdown
    pub confidence_breakdown: ConfidenceBreakdown,
    /// Alternative scenarios
    pub alternatives: Vec<AlternativeScenario>,
    /// Decision tree
    pub decision_path: Vec<String>,
}

impl RiskExplanation {
    /// Create empty explanation
    pub fn empty() -> Self {
        Self {
            summary: String::new(),
            explanations: Vec::new(),
            contributing_factors: Vec::new(),
            reducing_factors: Vec::new(),
            confidence_breakdown: ConfidenceBreakdown::default(),
            alternatives: Vec::new(),
            decision_path: Vec::new(),
        }
    }

    /// Create simple explanation from single statement
    pub fn single(category: &str, value: &str) -> Self {
        let mut exp = Self::empty();
        exp.add(category, value);
        exp.summary = value.to_string();
        exp
    }

    /// Add explanation entry
    pub fn add(&mut self, category: &str, value: &str) {
        self.explanations.push(ExplanationEntry {
            category: category.to_string(),
            value: value.to_string(),
            weight: Decimal::ONE,
        });
    }

    /// Add weighted explanation
    pub fn add_weighted(&mut self, category: &str, value: &str, weight: Decimal) {
        self.explanations.push(ExplanationEntry {
            category: category.to_string(),
            value: value.to_string(),
            weight,
        });
    }

    /// Add contributing factor
    pub fn add_contributor(&mut self, name: &str, impact: Decimal, description: &str) {
        self.contributing_factors.push(Factor {
            name: name.to_string(),
            impact,
            description: description.to_string(),
        });
    }

    /// Add reducing factor
    pub fn add_reducer(&mut self, name: &str, impact: Decimal, description: &str) {
        self.reducing_factors.push(Factor {
            name: name.to_string(),
            impact,
            description: description.to_string(),
        });
    }

    /// Format as readable string
    pub fn format(&self) -> String {
        let mut lines = vec![self.summary.clone(), String::new()];

        if !self.explanations.is_empty() {
            lines.push("## Decision Details".to_string());
            for entry in &self.explanations {
                lines.push(format!("- {}: {}", entry.category, entry.value));
            }
        }

        if !self.contributing_factors.is_empty() {
            lines.push(String::new());
            lines.push("## Contributors".to_string());
            for factor in &self.contributing_factors {
                lines.push(format!(
                    "+ {}: {} - {}",
                    factor.name, factor.impact, factor.description
                ));
            }
        }

        if !self.reducing_factors.is_empty() {
            lines.push(String::new());
            lines.push("## Reducing Factors".to_string());
            for factor in &self.reducing_factors {
                lines.push(format!(
                    "- {}: {} - {}",
                    factor.name, factor.impact, factor.description
                ));
            }
        }

        lines.join("\n")
    }
}

impl Default for RiskExplanation {
    fn default() -> Self {
        Self::empty()
    }
}

/// Single explanation entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplanationEntry {
    /// Category/name
    pub category: String,
    /// Explanation text
    pub value: String,
    /// Weight importance
    pub weight: Decimal,
}

/// Factor affecting decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Factor {
    /// Factor name
    pub name: String,
    /// Impact magnitude (0-1)
    pub impact: Decimal,
    /// Human-readable description
    pub description: String,
}

/// Confidence breakdown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceBreakdown {
    /// Signal confidence contribution
    pub signal: Decimal,
    /// Confluence contribution
    pub confluence: Decimal,
    /// Regime contribution
    pub regime: Decimal,
    /// Session contribution
    pub session: Decimal,
    /// Volatility contribution
    pub volatility: Decimal,
}

impl Default for ConfidenceBreakdown {
    fn default() -> Self {
        Self {
            signal: Decimal::from_f64(0.25).unwrap(),
            confluence: Decimal::from_f64(0.25).unwrap(),
            regime: Decimal::from_f64(0.20).unwrap(),
            session: Decimal::from_f64(0.15).unwrap(),
            volatility: Decimal::from_f64(0.15).unwrap(),
        }
    }
}

/// Alternative scenario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlternativeScenario {
    /// Scenario name
    pub name: String,
    /// Hypothetical result
    pub hypothetical_size: Decimal,
    /// What would need to change
    pub required_changes: Vec<String>,
    /// Probability of reaching this scenario
    pub probability: Decimal,
}

/// Trait for explainable components
pub trait Explainable {
    /// Generate explanation for this component's contribution
    fn explain(&self) -> RiskExplanation;
}

/// Explanation engine that builds comprehensive explanations
pub struct ExplanationEngine;

impl ExplanationEngine {
    /// Create new explanation engine
    pub fn new() -> Self {
        Self
    }

    /// Build explanation for position size decision
    pub fn build_position_size_explanation(
        &self,
        base_size: Decimal,
        adjustments: &[(String, Decimal)],
        final_size: Decimal,
    ) -> RiskExplanation {
        let mut exp = RiskExplanation::empty();

        exp.summary = format!(
            "Position size: {} lots (base: {}, adjustments applied)",
            final_size, base_size
        );

        exp.add("base_size", &format!("Base calculation: {} lots", base_size));

        for (name, multiplier) in adjustments {
            let change_pct = (*multiplier - Decimal::ONE) * Decimal::from(100);
            let direction = if *multiplier > Decimal::ONE {
                "+"
            } else {
                ""
            };
            exp.add(
                &format!("{}_adjustment", name),
                &format!("{}: {}{:.1}%", name, direction, change_pct),
            );
        }

        exp.add("final_size", &format!("Final: {} lots", final_size));

        exp
    }

    /// Build explanation for approval decision
    pub fn build_approval_explanation(
        &self,
        approved: bool,
        reasons: &[String],
        blocking_factors: &[String],
    ) -> RiskExplanation {
        let mut exp = RiskExplanation::empty();

        exp.summary = if approved {
            "Trade APPROVED".to_string()
        } else {
            "Trade REJECTED".to_string()
        };

        for reason in reasons {
            exp.add("approval_reason", reason);
        }

        for factor in blocking_factors {
            exp.add_reducer("blocking_factor", Decimal::ONE, factor);
        }

        exp
    }

    /// Build risk profile explanation
    pub fn build_profile_explanation(
        &self,
        profile: &crate::RiskProfile,
        confidence: Decimal,
        session_quality: Decimal,
    ) -> RiskExplanation {
        let mut exp = RiskExplanation::empty();

        exp.summary = format!("Risk profile: {:?}", profile);

        exp.add(
            "profile_selection",
            &format!("Selected {:?} based on confidence {:.1}% and session quality {:.1}%",
                profile, confidence * Decimal::from(100), session_quality * Decimal::from(100)),
        );

        exp.add_contributor(
            "confidence",
            confidence,
            &format!("Signal confidence at {:.1}%", confidence * Decimal::from(100)),
        );

        exp.add_contributor(
            "session",
            session_quality,
            &format!("Market session quality at {:.1}%", session_quality * Decimal::from(100)),
        );

        exp
    }

    /// Build comprehensive risk assessment explanation
    pub fn build_comprehensive(
        &self,
        result: &PositionSizeResult,
        drawdown: &DrawdownState,
        exposure: &ExposureMetrics,
        correlation: &Decimal,
        streak: &StreakState,
        vol: &VolatilityMetrics,
    ) -> RiskExplanation {
        let mut exp = RiskExplanation::empty();

        // Summary
        exp.summary = format!(
            "Risk assessment: {} lots ({:.2}% of capital) - {}",
            result.lot_size,
            result.risk_percent * Decimal::from(100),
            result.reasoning
        );

        // Position sizing breakdown
        exp.add("sizing_method", &format!("Method: {:?}", result.method));
        exp.add("lot_size", &format!("Lot size: {}", result.lot_size));
        exp.add(
            "risk_percent",
            &format!("Risk: {:.2}%", result.risk_percent * Decimal::from(100)),
        );
        exp.add(
            "capital_at_risk",
            &format!("Capital at risk: {}", result.capital_at_risk),
        );

        // Drawdown state
        match drawdown {
            DrawdownState::Normal => {}
            DrawdownState::Warning { pct } => {
                exp.add_reducer(
                    "drawdown_warning",
                    *pct,
                    &format!("Drawdown warning at {:.1}%", pct * Decimal::from(100)),
                );
            }
            DrawdownState::SoftLimit { pct } => {
                exp.add_reducer(
                    "drawdown_limit",
                    *pct,
                    &format!("Drawdown soft limit at {:.1}%", pct * Decimal::from(100)),
                );
            }
            DrawdownState::HardLimit => {
                exp.add_reducer("drawdown_block", Decimal::ONE, "Drawdown hard limit");
            }
            DrawdownState::RecoveryMode => {
                exp.add_reducer(
                    "recovery_mode",
                    Decimal::from_f64(0.5).unwrap(),
                    "In recovery mode",
                );
            }
        }

        // Exposure
        if exposure.total_positions > 5 {
            exp.add_reducer(
                "high_position_count",
                Decimal::from_f64(0.1).unwrap() * Decimal::from(exposure.total_positions as u32),
                &format!("{} positions already open", exposure.total_positions),
            );
        }

        // Correlation
        if *correlation > Decimal::from_f64(0.5).unwrap() {
            exp.add_reducer(
                "correlation",
                *correlation,
                &format!("Correlation score: {:.2}", correlation),
            );
        }

        // Streak
        if streak.consecutive_losses > 0 {
            exp.add_reducer(
                "losing_streak",
                Decimal::from(streak.consecutive_losses.min(5)) / Decimal::from(10),
                &format!("{} consecutive losses", streak.consecutive_losses),
            );
        } else if streak.consecutive_wins > 0 {
            exp.add_contributor(
                "winning_streak",
                Decimal::from(streak.consecutive_wins.min(3)) / Decimal::from(20),
                &format!("{} consecutive wins", streak.consecutive_wins),
            );
        }

        // Volatility
        match vol.regime {
            crate::volatility::VolatilityRegime::Low => {
                exp.add_contributor("low_volatility", Decimal::from_f64(0.1).unwrap(), "Low volatility");
            }
            crate::volatility::VolatilityRegime::High
            | crate::volatility::VolatilityRegime::VeryHigh => {
                exp.add_reducer(
                    "high_volatility",
                    Decimal::from_f64(0.2).unwrap(),
                    &format!("Regime: {:?}", vol.regime),
                );
            }
            _ => {}
        }

        exp
    }
}

impl Default for ExplanationEngine {
    fn default() -> Self {
        Self::new()
    }
}
