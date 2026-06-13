//! Market structure analysis

pub mod swings;
pub mod trend;
pub mod ranges;
pub mod impulse;
pub mod correction;

use crate::config::Config;
use crate::market_data::Candle;
use std::collections::HashMap;

/// Market structure analysis result
#[derive(Debug, Clone)]
pub struct StructureAnalysis {
    /// Swing highs detected
    pub swing_highs: Vec<swings::SwingPoint>,
    /// Swing lows detected
    pub swing_lows: Vec<swings::SwingPoint>,
    /// Current trend direction
    pub trend: trend::TrendDirection,
    /// Current range (if in one)
    pub range: Option<ranges::RangeStructure>,
}

/// Analyzes market structure across timeframes
#[derive(Debug)]
pub struct StructureAnalyzer {
    config: Config,
}

impl StructureAnalyzer {
    /// Create a new structure analyzer
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Analyze market structure from candle data
    pub fn analyze(&self, candles: &HashMap<String, Vec<Candle>>) -> crate::Result<StructureAnalysis> {
        // Analyze each timeframe
        let mut swing_highs = Vec::new();
        let mut swing_lows = Vec::new();

        for (timeframe, tf_candles) in candles {
            let sw = swings::detect_swings(tf_candles, self.config.swing_pivot_bars);
            swing_highs.extend(sw.highs);
            swing_lows.extend(sw.lows);
        }

        // Determine trend from H4 or H1
        let trend = if let Some(h4_candles) = candles.get("H4") {
            trend::classify_trend(h4_candles, &swing_highs, &swing_lows)
        } else if let Some(h1_candles) = candles.get("H1") {
            trend::classify_trend(h1_candles, &swing_highs, &swing_lows)
        } else {
            trend::TrendDirection::Undefined
        };

        // Detect range structure
        let range = ranges::detect_range(&swing_highs, &swing_lows);

        Ok(StructureAnalysis {
            swing_highs,
            swing_lows,
            trend,
            range,
        })
    }
}
