//! Promotion Module
//!
//! Manages the lifecycle and promotion paths for strategies from research
//! to production, enforcing strict requirements before advancing.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionState {
    Research,
    Sandbox,
    Shadow,
    Candidate,
    Production,
}

#[derive(Debug, Clone)]
pub struct PromotionRequirements {
    pub min_trade_count: usize,
    pub min_robustness_score: Decimal,
    pub min_oos_performance: Decimal,
    pub max_drawdown_limit: Decimal,
}

#[derive(Debug, Clone)]
pub struct PromotionDecision {
    pub strategy_id: String,
    pub from_state: PromotionState,
    pub to_state: PromotionState,
    pub is_approved: bool,
    pub reason: String,
}

pub struct PromotionEngine;

impl PromotionEngine {
    pub fn evaluate_promotion(
        _strategy_id: &str,
        _current_state: PromotionState,
        _requirements: &PromotionRequirements,
    ) -> Result<PromotionDecision, &'static str> {
        // Stub: evaluate if strategy meets all requirements for promotion
        Ok(PromotionDecision {
            strategy_id: _strategy_id.to_string(),
            from_state: _current_state.clone(),
            to_state: PromotionState::Sandbox, // Default next state stub
            is_approved: false,
            reason: "Insufficient data".to_string(),
        })
    }
}
