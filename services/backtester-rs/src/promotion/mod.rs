//! Promotion Module
//!
//! Manages the lifecycle and promotion paths for strategies from Research → Sandbox →
//! Shadow → Candidate → Production, enforcing real requirements before advancing.
//!
//! All decisions are computed from real performance metrics — no hardcoded outcomes.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromotionError {
    #[error("strategy ID must not be empty")]
    EmptyStrategyId,
    #[error("strategy is already at production state — cannot promote further")]
    AlreadyAtProduction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionState {
    Research,
    Sandbox,
    Shadow,
    Candidate,
    Production,
}

impl PromotionState {
    /// Return the next state in the promotion pipeline.
    pub fn next(&self) -> Option<PromotionState> {
        match self {
            PromotionState::Research => Some(PromotionState::Sandbox),
            PromotionState::Sandbox => Some(PromotionState::Shadow),
            PromotionState::Shadow => Some(PromotionState::Candidate),
            PromotionState::Candidate => Some(PromotionState::Production),
            PromotionState::Production => None,
        }
    }

    /// Human-readable display name.
    pub fn label(&self) -> &'static str {
        match self {
            PromotionState::Research => "Research",
            PromotionState::Sandbox => "Sandbox",
            PromotionState::Shadow => "Shadow",
            PromotionState::Candidate => "Candidate",
            PromotionState::Production => "Production",
        }
    }
}

/// Requirements that must be met for a strategy to advance to the next state.
#[derive(Debug, Clone)]
pub struct PromotionRequirements {
    /// Minimum number of completed trades in the current stage.
    pub min_trade_count: usize,
    /// Minimum robustness score (0.0–1.0) from walk-forward / out-of-sample analysis.
    pub min_robustness_score: Decimal,
    /// Minimum out-of-sample performance score (e.g. OOS expectancy ≥ threshold).
    pub min_oos_performance: Decimal,
    /// Maximum allowed drawdown fraction in the current stage.
    pub max_drawdown_limit: Decimal,
}

impl PromotionRequirements {
    /// Default production-quality requirements for Candidate → Production.
    pub fn production_grade() -> Self {
        Self {
            min_trade_count: 200,
            min_robustness_score: Decimal::new(65, 2), // 0.65
            min_oos_performance: Decimal::new(5, 2),   // 5% OOS expectancy
            max_drawdown_limit: Decimal::new(15, 2),   // 15% max DD
        }
    }

    /// Sandbox → Shadow requirements.
    pub fn shadow_grade() -> Self {
        Self {
            min_trade_count: 50,
            min_robustness_score: Decimal::new(50, 2),
            min_oos_performance: Decimal::new(2, 2),
            max_drawdown_limit: Decimal::new(20, 2),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromotionDecision {
    pub strategy_id: String,
    pub from_state: PromotionState,
    pub to_state: PromotionState,
    pub is_approved: bool,
    pub reason: String,
    /// Score relative to requirements (0.0 = just meets threshold, 1.0 = far exceeds).
    pub promotion_score: Decimal,
}

/// Performance snapshot used as input to the promotion evaluation.
#[derive(Debug, Clone)]
pub struct StrategyPerformanceSnapshot {
    pub trade_count: usize,
    pub robustness_score: Decimal,
    pub oos_performance: Decimal,
    pub max_drawdown: Decimal,
}

pub struct PromotionEngine;

impl PromotionEngine {
    /// Evaluate whether a strategy meets all requirements for promotion.
    ///
    /// Returns the promotion decision with the correct `to_state` (always the next
    /// stage in the pipeline) and `is_approved` based on real requirement checking.
    pub fn evaluate_promotion(
        strategy_id: &str,
        current_state: PromotionState,
        requirements: &PromotionRequirements,
        performance: &StrategyPerformanceSnapshot,
    ) -> Result<PromotionDecision, PromotionError> {
        if strategy_id.is_empty() {
            return Err(PromotionError::EmptyStrategyId);
        }

        let next_state = match current_state.next() {
            Some(s) => s,
            None => return Err(PromotionError::AlreadyAtProduction),
        };

        let mut failures: Vec<String> = Vec::new();

        if performance.trade_count < requirements.min_trade_count {
            failures.push(format!(
                "insufficient trades: {} < {}",
                performance.trade_count, requirements.min_trade_count
            ));
        }
        if performance.robustness_score < requirements.min_robustness_score {
            failures.push(format!(
                "robustness score {:.4} < minimum {:.4}",
                performance.robustness_score, requirements.min_robustness_score
            ));
        }
        if performance.oos_performance < requirements.min_oos_performance {
            failures.push(format!(
                "OOS performance {:.4} < minimum {:.4}",
                performance.oos_performance, requirements.min_oos_performance
            ));
        }
        if performance.max_drawdown > requirements.max_drawdown_limit {
            failures.push(format!(
                "drawdown {:.4} exceeds limit {:.4}",
                performance.max_drawdown, requirements.max_drawdown_limit
            ));
        }

        let is_approved = failures.is_empty();

        // Promotion score: composite margin above thresholds (normalised to [0, 1]).
        let promotion_score = if is_approved {
            let robustness_margin = (performance.robustness_score
                - requirements.min_robustness_score)
                / (Decimal::ONE - requirements.min_robustness_score).max(Decimal::new(1, 4));
            let oos_margin = (performance.oos_performance - requirements.min_oos_performance)
                / (Decimal::ONE - requirements.min_oos_performance).max(Decimal::new(1, 4));
            let dd_margin = (requirements.max_drawdown_limit - performance.max_drawdown)
                / requirements.max_drawdown_limit.max(Decimal::new(1, 4));
            ((robustness_margin + oos_margin + dd_margin) / Decimal::from(3i64))
                .max(Decimal::ZERO)
                .min(Decimal::ONE)
        } else {
            Decimal::ZERO
        };

        let reason = if is_approved {
            format!(
                "All requirements met. Promoting {} → {}.",
                current_state.label(),
                next_state.label()
            )
        } else {
            format!("Promotion denied: {}", failures.join("; "))
        };

        Ok(PromotionDecision {
            strategy_id: strategy_id.to_string(),
            from_state: current_state,
            to_state: next_state,
            is_approved,
            reason,
            promotion_score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meets_all(req: &PromotionRequirements) -> StrategyPerformanceSnapshot {
        StrategyPerformanceSnapshot {
            trade_count: req.min_trade_count + 50,
            robustness_score: req.min_robustness_score + Decimal::new(10, 2),
            oos_performance: req.min_oos_performance + Decimal::new(5, 2),
            max_drawdown: req.max_drawdown_limit - Decimal::new(5, 2),
        }
    }

    #[test]
    fn test_approved_promotion_research_to_sandbox() {
        let req = PromotionRequirements::shadow_grade();
        let perf = meets_all(&req);
        let decision = PromotionEngine::evaluate_promotion(
            "strategy_001",
            PromotionState::Research,
            &req,
            &perf,
        )
        .expect("ok");
        assert!(decision.is_approved);
        assert_eq!(decision.to_state, PromotionState::Sandbox);
        assert!(decision.promotion_score > Decimal::ZERO);
    }

    #[test]
    fn test_denied_insufficient_trades() {
        let req = PromotionRequirements::shadow_grade();
        let perf = StrategyPerformanceSnapshot {
            trade_count: 10, // below 50
            robustness_score: Decimal::new(70, 2),
            oos_performance: Decimal::new(8, 2),
            max_drawdown: Decimal::new(10, 2),
        };
        let decision = PromotionEngine::evaluate_promotion(
            "strategy_002",
            PromotionState::Sandbox,
            &req,
            &perf,
        )
        .expect("ok");
        assert!(!decision.is_approved);
        assert!(decision.reason.contains("insufficient trades"));
    }

    #[test]
    fn test_denied_drawdown_exceeds_limit() {
        let req = PromotionRequirements::production_grade();
        let perf = StrategyPerformanceSnapshot {
            trade_count: 300,
            robustness_score: Decimal::new(70, 2),
            oos_performance: Decimal::new(10, 2),
            max_drawdown: Decimal::new(25, 2), // exceeds 15% limit
        };
        let decision = PromotionEngine::evaluate_promotion(
            "strategy_003",
            PromotionState::Candidate,
            &req,
            &perf,
        )
        .expect("ok");
        assert!(!decision.is_approved);
        assert!(decision.reason.contains("drawdown"));
    }

    #[test]
    fn test_already_at_production_returns_error() {
        let req = PromotionRequirements::production_grade();
        let perf = meets_all(&req);
        let result = PromotionEngine::evaluate_promotion(
            "strategy_004",
            PromotionState::Production,
            &req,
            &perf,
        );
        assert!(matches!(result, Err(PromotionError::AlreadyAtProduction)));
    }

    #[test]
    fn test_correct_next_state_in_pipeline() {
        let req = PromotionRequirements::shadow_grade();
        let perf = meets_all(&req);
        let states = [
            (PromotionState::Research, PromotionState::Sandbox),
            (PromotionState::Sandbox, PromotionState::Shadow),
            (PromotionState::Shadow, PromotionState::Candidate),
            (PromotionState::Candidate, PromotionState::Production),
        ];
        for (from, expected_to) in states {
            let decision =
                PromotionEngine::evaluate_promotion("strategy_005", from, &req, &perf).expect("ok");
            assert_eq!(decision.to_state, expected_to);
        }
    }
}
