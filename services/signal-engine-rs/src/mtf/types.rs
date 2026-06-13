//! MTF analysis types

use serde::{Deserialize, Serialize};

/// Multi-timeframe alignment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTFAlignmentResult {
    /// Whether timeframes are aligned
    pub aligned: bool,
    /// Reference (highest) timeframe
    pub reference_timeframe: String,
    /// Individual timeframe alignments
    pub alignments: Vec<TimeframeAlignment>,
    /// Overall alignment score (0.0 - 1.0)
    pub alignment_score: f64,
    /// Overall market bias
    pub bias: MarketBias,
}

/// Alignment for a specific timeframe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeAlignment {
    /// Timeframe name
    pub timeframe: String,
    /// Direction of this timeframe
    pub direction: AlignmentDirection,
    /// Weight in overall alignment
    pub weight: f64,
    /// Context description
    pub context: String,
}

/// Market direction from alignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlignmentDirection {
    /// Bullish alignment
    Bullish,
    /// Bearish alignment
    Bearish,
    /// Neutral/uncertain
    Neutral,
    /// Conflicting with other timeframes
    Conflict,
}

/// Overall market bias
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketBias {
    /// Strong bullish bias
    StrongBullish,
    /// Bullish bias
    Bullish,
    /// Neutral
    Neutral,
    /// Bearish bias
    Bearish,
    /// Strong bearish bias
    StrongBearish,
}

impl MarketBias {
    /// Check if bias supports long trades
    pub fn allows_long(&self) -> bool {
        matches!(self, MarketBias::StrongBullish | MarketBias::Bullish)
    }

    /// Check if bias supports short trades
    pub fn allows_short(&self) -> bool {
        matches!(self, MarketBias::StrongBearish | MarketBias::Bearish)
    }

    /// Check if bias conflicts with direction
    pub fn conflicts_with(&self, direction: TradeSide) -> bool {
        match (self, direction) {
            (MarketBias::StrongBearish | MarketBias::Bearish, TradeSide::Buy) => true,
            (MarketBias::StrongBullish | MarketBias::Bullish, TradeSide::Sell) => true,
            _ => false,
        }
    }
}

/// Trade side
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeSide {
    /// Buy/Long
    Buy,
    /// Sell/Short
    Sell,
}
