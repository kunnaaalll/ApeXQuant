use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use crate::market_data_integration::{MarketConfidenceScore, MarketWarningLevel};
use crate::strategy_integration::{StrategyRanking, StrategyScalingRecommendation, StrategyRetirementRecommendation};
use crate::execution_integration::ExecutionConfidence;
use crate::risk_integration::{RiskApprovalScore, AllocationLimits, ScalingPermissions};
use crate::portfolio_integration::{AllocationChanges, CapitalRotationSuggestions};
use crate::learning_integration::{ResearchRequests, OptimizationRequests, DiscoveryRequests};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FinalRecommendationPackage {
    pub package_id: Uuid,
    pub timestamp: u64,
    pub strategy_id: Option<Uuid>,
    pub unified_confidence: Decimal,
    pub scaling_action: Option<StrategyScalingRecommendation>,
    pub allocation_change: Option<AllocationChanges>,
    pub rotation_suggestion: Option<CapitalRotationSuggestions>,
    pub research_request: Option<ResearchRequests>,
    pub optimization_request: Option<OptimizationRequests>,
    pub discovery_request: Option<DiscoveryRequests>,
    pub retirement_recommendation: Option<StrategyRetirementRecommendation>,
    pub warnings: Vec<MarketWarningLevel>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrioritizedDecision {
    pub priority_score: u32,
    pub package: FinalRecommendationPackage,
}

impl Ord for PrioritizedDecision {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_score.cmp(&other.priority_score)
    }
}

impl PartialOrd for PrioritizedDecision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct DecisionPriorityQueue {
    queue: BinaryHeap<PrioritizedDecision>,
}

impl DecisionPriorityQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, decision: PrioritizedDecision) {
        self.queue.push(decision);
    }

    pub fn pop(&mut self) -> Option<PrioritizedDecision> {
        self.queue.pop()
    }
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn resolve(
        scaling: &Option<StrategyScalingRecommendation>,
        permissions: &ScalingPermissions,
    ) -> Option<StrategyScalingRecommendation> {
        match scaling {
            Some(StrategyScalingRecommendation::ScaleUp) if !permissions.can_scale_up => {
                Some(StrategyScalingRecommendation::Maintain)
            }
            Some(StrategyScalingRecommendation::ScaleDown) if !permissions.can_scale_down => {
                Some(StrategyScalingRecommendation::Maintain)
            }
            other => other.clone(),
        }
    }
}

pub struct DecisionMerger;

impl DecisionMerger {
    pub fn merge(
        market_conf: &MarketConfidenceScore,
        strategy_conf: &Decimal, // From StrategyRanking or separate
        execution_conf: &ExecutionConfidence,
        risk_approval: &RiskApprovalScore,
    ) -> Decimal {
        // Simple weighted average, risk approval can veto
        if risk_approval.score == Decimal::new(0, 0) {
            return Decimal::new(0, 0);
        }
        
        // (market_conf * 0.3 + strategy_conf * 0.4 + execution_conf * 0.3) * risk_approval
        let w_market = market_conf.score * Decimal::new(30, 2);
        let w_strategy = strategy_conf * Decimal::new(40, 2);
        let w_exec = execution_conf.confidence_score * Decimal::new(30, 2);

        (w_market + w_strategy + w_exec) * risk_approval.score
    }
}

pub struct DecisionPipeline;

impl DecisionPipeline {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process_cycle(
        &self,
        strategy_id: Uuid,
        market_conf: MarketConfidenceScore,
        market_warn: MarketWarningLevel,
        strategy_rank: StrategyRanking,
        strategy_scale: StrategyScalingRecommendation,
        strategy_retire: StrategyRetirementRecommendation,
        exec_conf: ExecutionConfidence,
        risk_score: RiskApprovalScore,
        risk_perms: ScalingPermissions,
        _alloc_limits: AllocationLimits,
    ) -> FinalRecommendationPackage {
        
        let unified_confidence = DecisionMerger::merge(
            &market_conf,
            &(strategy_rank.total_score),
            &exec_conf,
            &risk_score,
        );

        let resolved_scaling = ConflictResolver::resolve(&Some(strategy_scale), &risk_perms);

        let mut warnings = Vec::new();
        if market_warn != MarketWarningLevel::None {
            warnings.push(market_warn);
        }

        FinalRecommendationPackage {
            package_id: Uuid::new_v4(),
            timestamp: 0, // Placeholder
            strategy_id: Some(strategy_id),
            unified_confidence,
            scaling_action: resolved_scaling,
            allocation_change: None,
            rotation_suggestion: None,
            research_request: None,
            optimization_request: None,
            discovery_request: None,
            retirement_recommendation: Some(strategy_retire),
            warnings,
        }
    }
}
