//! MTF alignment calculation

use crate::market_data::Candle;
use crate::mtf::hierarchy::TimeframeHierarchy;
use crate::mtf::types::{AlignmentDirection, MTFAlignmentResult, MarketBias, TimeframeAlignment};
use crate::structure::{StructureAnalysis, trend::TrendDirection};
use std::collections::HashMap;

/// Analyzes alignment across timeframes
#[derive(Debug)]
pub struct MTFAligner<'a> {
    hierarchy: &'a TimeframeHierarchy,
}

impl<'a> MTFAligner<'a> {
    /// Create a new aligner
    pub fn new(hierarchy: &'a TimeframeHierarchy) -> Self {
        Self { hierarchy }
    }

    /// Analyze alignment across all timeframes
    pub fn analyze(
        &self,
        candles: &HashMap<String, Vec<Candle>>,
        structure: &StructureAnalysis,
    ) -> crate::Result<MTFAlignmentResult> {
        let mut alignments = Vec::new();
        let mut bullish_weight = 0.0;
        let mut bearish_weight = 0.0;

        for tf in self.hierarchy.ordered() {
            let weight = self.hierarchy.weight(tf);
            let direction = self.determine_tf_direction(tf, structure);

            match direction {
                AlignmentDirection::Bullish => bullish_weight += weight,
                AlignmentDirection::Bearish => bearish_weight += weight,
                _ => {}
            }

            alignments.push(TimeframeAlignment {
                timeframe: tf.clone(),
                direction,
                weight,
                context: self.describe_tf_context(tf, structure),
            });
        }

        let total_weight = bullish_weight + bearish_weight;
        let alignment_score = if total_weight > 0.0 {
            (bullish_weight.max(bearish_weight)) / total_weight
        } else {
            0.0
        };

        let aligned = alignment_score > 0.7; // 70% threshold

        let bias = if bullish_weight > bearish_weight * 3.0 {
            MarketBias::StrongBullish
        } else if bullish_weight > bearish_weight * 1.5 {
            MarketBias::Bullish
        } else if bearish_weight > bullish_weight * 3.0 {
            MarketBias::StrongBearish
        } else if bearish_weight > bullish_weight * 1.5 {
            MarketBias::Bearish
        } else {
            MarketBias::Neutral
        };

        Ok(MTFAlignmentResult {
            aligned,
            reference_timeframe: self.hierarchy.highest().cloned().unwrap_or_default(),
            alignments,
            alignment_score,
            bias,
        })
    }

    /// Determine direction for a specific timeframe
    fn determine_tf_direction(&self, timeframe: &str, structure: &StructureAnalysis) -> AlignmentDirection {
        // Use structure trend for higher timeframes
        match timeframe {
            "H4" | "Daily" | "Weekly" => {
                match structure.trend {
                    TrendDirection::Up => AlignmentDirection::Bullish,
                    TrendDirection::Down => AlignmentDirection::Bearish,
                    TrendDirection::Sideways => AlignmentDirection::Neutral,
                    TrendDirection::Undefined => AlignmentDirection::Neutral,
                }
            }
            _ => {
                // Lower timeframes - check for CHoCH/BOS signals
                // Simplified - would check actual pattern presence
                match structure.trend {
                    TrendDirection::Up => AlignmentDirection::Bullish,
                    TrendDirection::Down => AlignmentDirection::Bearish,
                    _ => AlignmentDirection::Neutral,
                }
            }
        }
    }

    /// Describe the context for a timeframe
    fn describe_tf_context(&self, timeframe: &str, structure: &StructureAnalysis) -> String {
        match timeframe {
            "H4" => match structure.trend {
                TrendDirection::Up => "bullish_structure".to_string(),
                TrendDirection::Down => "bearish_structure".to_string(),
                _ => "choppy_structure".to_string(),
            },
            "H1" => {
                if structure.swing_highs.len() >= 2 && structure.swing_lows.len() >= 2 {
                    "established_swings".to_string()
                } else {
                    "forming_structure".to_string()
                }
            }
            _ => "continuation".to_string(),
        }
    }
}
