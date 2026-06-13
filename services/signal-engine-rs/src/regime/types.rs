//! Market regime types

use serde::{Deserialize, Serialize};

/// Market regime classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegime {
    /// Type of regime
    pub regime_type: RegimeType,
    /// Confidence in classification (0.0 - 1.0)
    pub confidence: f64,
    /// Current volatility percentile (0.0 - 1.0)
    pub volatility_percentile: f64,
    /// Trend strength (0.0 - 1.0)
    pub trend_strength: f64,
}

/// Regime type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegimeType {
    /// Strong uptrend
    TrendingUp,
    /// Strong downtrend
    TrendingDown,
    /// Ranging/oscillating
    Ranging,
    /// Breakout forming
    Breakout,
    /// High volatility
    HighVolatility,
    /// Low volatility
    LowVolatility,
    /// Transition phase
    Transition,
    /// Undefined
    Undefined,
}

impl RegimeType {
    /// Check if regime supports breakout strategies
    pub fn supports_breakout(&self) -> bool {
        matches!(
            self,
            RegimeType::Ranging | RegimeType::Breakout | RegimeType::LowVolatility
        )
    }

    /// Check if regime supports trend following
    pub fn supports_trend(&self) -> bool {
        matches!(self, RegimeType::TrendingUp | RegimeType::TrendingDown)
    }

    /// Check if regime requires caution
    pub fn requires_caution(&self) -> bool {
        matches!(
            self,
            RegimeType::HighVolatility | RegimeType::Transition | RegimeType::Undefined
        )
    }
}
