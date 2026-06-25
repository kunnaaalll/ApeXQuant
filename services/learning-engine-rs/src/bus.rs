use crate::decay::DecayOutput;
use crate::discovery::DiscoveryResult;
use crate::recommendation::LearningRecommendation;
use crate::adaptation::AdaptationResult;
use crate::retirement::RetirementAction;
use crate::promotion::PromotionLevel;
use crate::optimization::OptimizationResult;
use crate::anomaly::AnomalyReport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncomingEvent {
    ExecutionFill,
    ExecutionSlippage,
    ExecutionRejection,
    StrategySignal,
    StrategyHealth,
    MarketIntelligence,
    MarketRegime,
    RiskAlert,
    PortfolioUpdate,
    PortfolioDrawdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutgoingEvent {
    LearningInsight(String),
    LearningRecommendation(LearningRecommendation),
    LearningDecay(DecayOutput),
    LearningDiscovery(DiscoveryResult),
    LearningAdaptation(AdaptationResult),
    LearningRetirement(RetirementAction, String), // Strategy ID
    LearningPromotion(PromotionLevel, String),    // Strategy ID
    LearningOptimization(OptimizationResult),
    LearningAnomaly(AnomalyReport),
}

pub struct EventBusIntegration {
    // Stub for connection
}

impl Default for EventBusIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBusIntegration {
    pub fn new() -> Self {
        Self {}
    }

    pub fn consume(&self, _event: IncomingEvent) {
        // Integrate with event bus (Redis/Kafka) here
    }

    pub fn publish(&self, _event: OutgoingEvent) {
        // Integrate with event bus (Redis/Kafka) here
    }
}
