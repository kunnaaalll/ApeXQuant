//! Market structure analysis

pub mod correction;
pub mod impulse;
pub mod ranges;
pub mod swings;
pub mod trend;

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
    pub fn analyze(
        &self,
        candles: &HashMap<String, Vec<Candle>>,
    ) -> crate::Result<StructureAnalysis> {
        let mut swing_highs = Vec::new();
        let mut swing_lows = Vec::new();
        let mut tf_swings: HashMap<String, (Vec<swings::SwingPoint>, Vec<swings::SwingPoint>)> = HashMap::new();

        for (timeframe, tf_candles) in candles {
            let sw = swings::detect_swings(tf_candles, self.config.swing_pivot_bars);
            tf_swings.insert(timeframe.clone(), (sw.highs.clone(), sw.lows.clone()));
            swing_highs.extend(sw.highs);
            swing_lows.extend(sw.lows);
        }

        // Determine trend from available timeframes: H4 -> H1 -> M15 -> M5 -> M1
        let timeframes_to_try = ["H4", "H1", "M15", "M5", "M1"];
        let mut trend = trend::TrendDirection::Undefined;

        for tf in timeframes_to_try {
            if let Some(tf_candles) = candles.get(tf) {
                let (sh, sl) = tf_swings.get(tf).cloned().unwrap_or_default();
                let detected = trend::classify_trend(tf_candles, &sh, &sl);
                if detected != trend::TrendDirection::Undefined {
                    trend = detected;
                    break;
                }
            }
        }

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
