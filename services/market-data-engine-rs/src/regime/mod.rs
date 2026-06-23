#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketRegime {
    Trending,
    Ranging,
    Expansion,
    Contraction,
    Transition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegimeConfidence(pub u32);

pub struct RegimeMetrics {
    pub regime: MarketRegime,
    pub confidence: RegimeConfidence,
}

pub struct RegimeEngine;

impl RegimeEngine {
    pub fn detect(trend_strength: crate::trend::TrendStrength, vol_grade: crate::volatility::VolatilityGrade) -> Result<RegimeMetrics, &'static str> {
        use crate::trend::TrendStrength;
        use crate::volatility::VolatilityGrade;

        let (regime, conf) = match (trend_strength, vol_grade) {
            (TrendStrength::Extreme, VolatilityGrade::Extreme) | (TrendStrength::Strong, VolatilityGrade::High) => (MarketRegime::Expansion, 90),
            (TrendStrength::Strong, _) | (TrendStrength::Extreme, _) => (MarketRegime::Trending, 80),
            (TrendStrength::Weak, VolatilityGrade::VeryLow) | (TrendStrength::Weak, VolatilityGrade::Low) => (MarketRegime::Contraction, 85),
            (TrendStrength::Normal, VolatilityGrade::Normal) => (MarketRegime::Ranging, 70),
            _ => (MarketRegime::Transition, 50),
        };

        Ok(RegimeMetrics {
            regime,
            confidence: RegimeConfidence(conf.clamp(0, 100)),
        })
    }
}
