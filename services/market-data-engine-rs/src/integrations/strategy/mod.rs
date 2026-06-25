use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyIntelligenceSnapshot {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub regime: RegimeSnapshot,
    pub opportunity: OpportunitySnapshot,
    pub health: SymbolHealthSnapshot,
    pub confidence: ConfidenceInputs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeSnapshot {
    pub state: String,
    pub probability: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunitySnapshot {
    pub score: Decimal,
    pub expected_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolHealthSnapshot {
    pub is_healthy: bool,
    pub data_quality_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInputs {
    pub market_confidence: Decimal,
    pub model_confidence: Decimal,
}
