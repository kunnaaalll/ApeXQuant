use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketRegimeType {
    TrendingUp,
    TrendingDown,
    Ranging,
    HighVolatility,
    LowVolatility,
    Expansion,
    Compression,
    Accumulation,
    Distribution,
    LiquidityCrisis,
    Transition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeClassification {
    pub regime_type: MarketRegimeType,
    pub confidence: Decimal,
    pub duration_ticks: u64,
}

pub struct RegimeClassifier;

impl RegimeClassifier {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn classify(
        &self,
        price_change: Decimal,
        volatility: Decimal,
        volume: Decimal,
    ) -> RegimeClassification {
        // Deterministic logic based on simple thresholds
        let volatility_threshold = Decimal::new(2, 0); // 2.0
        let trend_threshold = Decimal::new(5, 1); // 0.5

        let mut regime = MarketRegimeType::Ranging;
        let mut confidence = Decimal::new(50, 2); // 0.50

        if volatility > volatility_threshold {
            if volume > Decimal::new(10000, 0) {
                regime = MarketRegimeType::LiquidityCrisis;
                confidence = Decimal::new(95, 2); // 0.95
            } else {
                regime = MarketRegimeType::HighVolatility;
                confidence = Decimal::new(80, 2); // 0.80
            }
        } else if price_change > trend_threshold {
            regime = MarketRegimeType::TrendingUp;
            confidence = Decimal::new(85, 2); // 0.85
        } else if price_change < -trend_threshold {
            regime = MarketRegimeType::TrendingDown;
            confidence = Decimal::new(85, 2); // 0.85
        }

        RegimeClassification {
            regime_type: regime,
            confidence,
            duration_ticks: 1,
        }
    }
}
