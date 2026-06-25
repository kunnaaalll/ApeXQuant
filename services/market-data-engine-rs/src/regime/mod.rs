use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketRegime {
    TrendFollowing,
    MeanReversion,
    Breakout,
    Expansion,
    Compression,
    RiskOn,
    RiskOff,
    Transitional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegimeMetrics {
    pub current_regime: MarketRegime,
    pub confidence_score: u8,
}

#[derive(Debug, Clone)]
pub struct RegimeEngine {
    // Inputs from other engines would be required in a real scenario
    // For deterministic testing, we just maintain a simple state proxy
    historical_regimes: Vec<MarketRegime>,
    persistence: u32,
}

impl Default for RegimeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RegimeEngine {
    pub fn new() -> Self {
        Self {
            historical_regimes: Vec::with_capacity(100),
            persistence: 0,
        }
    }

    pub fn determine_regime(
        &mut self,
        volatility_expanding: bool,
        volatility_contracting: bool,
        trend_strength: u8,
        structure_state: crate::structure::StructureState,
    ) -> Result<RegimeMetrics, &'static str> {
        
        let mut determined = MarketRegime::Transitional;
        let mut confidence = Decimal::ZERO;

        if volatility_expanding && structure_state == crate::structure::StructureState::Trending {
            determined = MarketRegime::TrendFollowing;
            confidence = Decimal::from(trend_strength);
        } else if volatility_expanding && structure_state == crate::structure::StructureState::Transition {
            determined = MarketRegime::Breakout;
            confidence = Decimal::from(80);
        } else if volatility_contracting && structure_state == crate::structure::StructureState::Compression {
            determined = MarketRegime::Compression;
            confidence = Decimal::from(90);
        } else if structure_state == crate::structure::StructureState::Ranging {
            determined = MarketRegime::MeanReversion;
            confidence = Decimal::from(100 - trend_strength);
        } else if volatility_expanding {
            determined = MarketRegime::Expansion;
            confidence = Decimal::from(70);
        }

        // Apply persistence smoothing
        let last = self.historical_regimes.last().copied().unwrap_or(MarketRegime::Transitional);
        if last == determined {
            self.persistence += 1;
            confidence += Decimal::from(self.persistence);
        } else {
            self.persistence = 1;
        }

        self.historical_regimes.push(determined);
        if self.historical_regimes.len() > 100 {
            self.historical_regimes.remove(0);
        }

        let final_confidence = confidence.to_u8().unwrap_or(0).min(100);

        Ok(RegimeMetrics {
            current_regime: determined,
            confidence_score: final_confidence,
        })
    }
}
