use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadProfile {
    pub current_spread: Decimal,
    pub average_spread: Decimal,
    pub variance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageRiskProfile {
    pub expected_slippage: Decimal,
    pub max_slippage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityProfile {
    pub bid_depth: Decimal,
    pub ask_depth: Decimal,
    pub imbalance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpactProfile {
    pub estimated_impact: Decimal,
    pub impact_decay: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSuitabilityScore {
    pub score: Decimal,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLiquidityProfile {
    pub session_name: String,
    pub historical_volume: Decimal,
    pub expected_volume: Decimal,
}
