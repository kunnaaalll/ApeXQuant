use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityProfile {
    pub realized: Decimal,
    pub implied: Option<Decimal>,
    pub forecast: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationProfile {
    pub market_correlation: Decimal,
    pub sector_correlation: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressIndicators {
    pub value_at_risk: Decimal,
    pub expected_shortfall: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureSignals {
    pub current_exposure: Decimal,
    pub max_recommended_exposure: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityWarnings {
    pub is_illiquid: bool,
    pub depth_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketShockSignals {
    pub shock_probability: Decimal,
    pub severity_estimate: Decimal,
}
