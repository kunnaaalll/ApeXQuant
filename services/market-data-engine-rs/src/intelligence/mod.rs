use crate::volatility::VolatilityMetrics;
use crate::trend::TrendMetrics;
use crate::momentum::MomentumMetrics;
use crate::structure::StructureMetrics;
use crate::correlation::CorrelationMetrics;
use crate::regime::RegimeMetrics;
use crate::quality::MarketQualityMetrics;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketIntelligenceProfile {
    pub volatility: VolatilityMetrics,
    pub trend: TrendMetrics,
    pub momentum: MomentumMetrics,
    pub structure: StructureMetrics,
    pub correlation: CorrelationMetrics,
    pub regime: RegimeMetrics,
    pub quality: MarketQualityMetrics,
    pub market_score: u8,
    pub confidence_score: u8,
    pub tradeability_score: u8,
}

pub struct IntelligenceAggregator;

impl IntelligenceAggregator {
    pub fn build_profile(
        volatility: VolatilityMetrics,
        trend: TrendMetrics,
        momentum: MomentumMetrics,
        structure: StructureMetrics,
        correlation: CorrelationMetrics,
        regime: RegimeMetrics,
        quality: MarketQualityMetrics,
    ) -> Result<MarketIntelligenceProfile, &'static str> {
        
        // Example deterministic aggregation logic bounds checked 0-100
        
        let market_score_dec = Decimal::from(trend.strength_score) * rust_decimal::prelude::FromStr::from_str("0.4").unwrap_or(Decimal::ZERO) 
            + Decimal::from(momentum.grade as u8 * 20) * rust_decimal::prelude::FromStr::from_str("0.3").unwrap_or(Decimal::ZERO)
            + Decimal::from(volatility.score) * rust_decimal::prelude::FromStr::from_str("0.3").unwrap_or(Decimal::ZERO);
            
        let market_score = market_score_dec.to_u8().unwrap_or(0).min(100);

        let confidence_score = regime.confidence_score;

        let tradeability_score = quality.overall_score;

        Ok(MarketIntelligenceProfile {
            volatility,
            trend,
            momentum,
            structure,
            correlation,
            regime,
            quality,
            market_score,
            confidence_score,
            tradeability_score,
        })
    }
}
