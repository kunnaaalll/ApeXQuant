use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EdgeDecay {
    pub strategy_id: Uuid,
    pub decay_rate: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegimeMemory {
    pub regime: String,
    pub successful_strategies: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionEvents {
    pub strategy_id: Uuid,
    pub from_stage: String,
    pub to_stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetirementEvents {
    pub strategy_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchResults {
    pub request_id: Uuid,
    pub result_summary: String,
    pub is_successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchRequests {
    pub request_id: Uuid,
    pub target_regime: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptimizationRequests {
    pub strategy_id: Uuid,
    pub target_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryRequests {
    pub market_id: String,
    pub timeframe: String,
}

pub struct LearningIntegration;

impl LearningIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_learning(
        decay: &EdgeDecay,
        _memory: &RegimeMemory,
    ) -> (Vec<ResearchRequests>, Vec<OptimizationRequests>, Vec<DiscoveryRequests>) {
        let mut opt_reqs = Vec::new();
        let mut research_reqs = Vec::new();

        if decay.decay_rate > Decimal::new(20, 2) {
            opt_reqs.push(OptimizationRequests {
                strategy_id: decay.strategy_id,
                target_metric: "SharpeRatio".to_string(),
            });
        }

        if decay.decay_rate > Decimal::new(50, 2) {
            research_reqs.push(ResearchRequests {
                request_id: Uuid::new_v4(),
                target_regime: "Current".to_string(),
                priority: 1,
            });
        }

        (research_reqs, opt_reqs, vec![])
    }
}
