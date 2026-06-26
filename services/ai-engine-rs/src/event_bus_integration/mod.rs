use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsumerEvent {
    MarketIntelligence { data: String },
    StrategyEvent { strategy_id: Uuid, event_type: String },
    ExecutionEvent { order_id: Uuid, event_type: String },
    RiskEvent { risk_topic: String, severity: String }, // New in Phase 4
    PortfolioEvent { portfolio_id: Uuid, event_type: String },
    LearningEvent { topic: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PublisherEvent {
    AiRecommendation { id: Uuid, topic: String },
    AiWarning { id: Uuid, message: String },
    AiResearchRequest { request_id: Uuid, target: String },
    AiOptimizationRequest { strategy_id: Uuid, target_metric: String }, // New in Phase 4
    AiAllocationChange { strategy_id: Uuid, new_allocation: Decimal },
}
