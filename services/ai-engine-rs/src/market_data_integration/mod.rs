use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketIntelligenceProfile {
    pub profile_id: Uuid,
    pub timestamp: u64,
    pub regime_state: RegimeState,
    pub volatility_state: VolatilityState,
    pub liquidity_profile: LiquidityProfile,
    pub correlation_state: CorrelationState,
    pub session_state: SessionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegimeState {
    Bull,
    Bear,
    Ranging,
    Breakout,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolatilityState {
    pub current_volatility: Decimal,
    pub historical_volatility: Decimal,
    pub regime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiquidityProfile {
    pub bid_ask_spread: Decimal,
    pub order_book_depth: Decimal,
    pub market_impact_estimate: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrelationState {
    pub average_correlation: Decimal,
    pub sector_correlation: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionState {
    PreMarket,
    RegularTrading,
    PostMarket,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketConfidenceScore {
    pub score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketOpportunityScore {
    pub score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketWarningLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

pub struct MarketDataIntegration;

impl MarketDataIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_market(profile: &MarketIntelligenceProfile) -> (MarketConfidenceScore, MarketOpportunityScore, MarketWarningLevel) {
        // Deterministic generation
        let confidence = if profile.regime_state == RegimeState::Unknown {
            Decimal::new(50, 2) // 0.50
        } else {
            Decimal::new(85, 2) // 0.85
        };

        let opportunity = Decimal::new(70, 2); // 0.70

        let warning = if profile.volatility_state.current_volatility > Decimal::new(100, 0) {
            MarketWarningLevel::High
        } else {
            MarketWarningLevel::None
        };

        (
            MarketConfidenceScore { score: confidence },
            MarketOpportunityScore { score: opportunity },
            warning
        )
    }
}
