use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use super::clusters::PortfolioCorrelationResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRegime {
    Trending,
    Ranging,
    HighVolatility,
    LowVolatility,
    Crash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationEvent {
    PositionChanged { timestamp: time::OffsetDateTime, symbol: String, new_size: Decimal },
    RegimeChanged { timestamp: time::OffsetDateTime, new_regime: MarketRegime },
    VolatilitySpikeDetected { timestamp: time::OffsetDateTime, symbol: String, z_score: Decimal },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationSnapshot {
    pub version: u64,
    pub timestamp: time::OffsetDateTime,
    pub regime: MarketRegime,
    pub result: PortfolioCorrelationResult,
}
