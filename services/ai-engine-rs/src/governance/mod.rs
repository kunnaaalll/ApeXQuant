use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionStage {
    Experimental,
    Shadow,
    Candidate,
    Production,
    Certified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConstraints {
    pub minimum_sample_size: u64,
    pub maximum_drift: Decimal,
    pub confidence_threshold: Decimal,
    pub replay_validation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRequest {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub current_stage: PromotionStage,
    pub requested_stage: PromotionStage,
    pub constraints_met: bool,
}

// Phase 4 additions: Institutional controls for AI Engine

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalControls {
    // Actions AI CANNOT do:
    pub can_override_risk_freezes: bool, // must be false
    pub can_increase_leverage: bool,     // must be false
    pub can_disable_protections: bool,   // must be false
    pub can_bypass_portfolio_limits: bool, // must be false
    pub can_enable_uncertified_strategies: bool, // must be false
    
    // Actions AI MAY do:
    pub can_recommend: bool,             // must be true
    pub can_rank: bool,                  // must be true
    pub can_prioritize: bool,            // must be true
    pub can_request_research: bool,      // must be true
    pub can_suggest_allocation_changes: bool, // must be true
}

impl Default for InstitutionalControls {
    fn default() -> Self {
        Self {
            can_override_risk_freezes: false,
            can_increase_leverage: false,
            can_disable_protections: false,
            can_bypass_portfolio_limits: false,
            can_enable_uncertified_strategies: false,
            can_recommend: true,
            can_rank: true,
            can_prioritize: true,
            can_request_research: true,
            can_suggest_allocation_changes: true,
        }
    }
}

pub struct GovernanceEngine;

impl GovernanceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_action(action_type: &str, controls: &InstitutionalControls) -> bool {
        match action_type {
            "override_risk_freezes" => controls.can_override_risk_freezes,
            "increase_leverage" => controls.can_increase_leverage,
            "disable_protections" => controls.can_disable_protections,
            "bypass_portfolio_limits" => controls.can_bypass_portfolio_limits,
            "enable_uncertified_strategies" => controls.can_enable_uncertified_strategies,
            "recommend" => controls.can_recommend,
            "rank" => controls.can_rank,
            "prioritize" => controls.can_prioritize,
            "request_research" => controls.can_request_research,
            "suggest_allocation_changes" => controls.can_suggest_allocation_changes,
            _ => false, // Default deny
        }
    }
}
