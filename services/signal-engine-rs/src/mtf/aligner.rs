//! MTF alignment calculation

use crate::market_data::Candle;
use crate::mtf::hierarchy::TimeframeHierarchy;
use crate::mtf::types::{AlignmentDirection, MTFAlignmentResult, MarketBias, TimeframeAlignment};
use crate::structure::{trend::TrendDirection, StructureAnalysis};
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
            let direction = self.determine_tf_direction(tf, candles, structure);

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

        let bias = if bullish_weight > bearish_weight {
            if bullish_weight >= 0.5 {
                MarketBias::StrongBullish
            } else {
                MarketBias::Bullish
            }
        } else if bearish_weight > bullish_weight {
            if bearish_weight >= 0.5 {
                MarketBias::StrongBearish
            } else {
                MarketBias::Bearish
            }
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
    fn determine_tf_direction(
        &self,
        timeframe: &str,
        candles: &HashMap<String, Vec<Candle>>,
        structure: &StructureAnalysis,
    ) -> AlignmentDirection {
        // 1. Try direct timeframe candles
        if let Some(tf_candles) = candles.get(timeframe) {
            if let Some(last) = tf_candles.last() {
                if tf_candles.len() >= 2 {
                    let first = &tf_candles[0];
                    if last.close > first.close {
                        return AlignmentDirection::Bullish;
                    } else if last.close < first.close {
                        return AlignmentDirection::Bearish;
                    }
                }
                // Single candle check (close vs open)
                if last.close >= last.open {
                    return AlignmentDirection::Bullish;
                } else {
                    return AlignmentDirection::Bearish;
                }
            }
        }

        // 2. Check structure trend
        match structure.trend {
            TrendDirection::Up => return AlignmentDirection::Bullish,
            TrendDirection::Down => return AlignmentDirection::Bearish,
            _ => {}
        }

        // 3. Fallback: Check ANY available timeframe candles in the map
        for (_tf_key, tf_candles) in candles {
            if let Some(last) = tf_candles.last() {
                if last.close >= last.open {
                    return AlignmentDirection::Bullish;
                } else {
                    return AlignmentDirection::Bearish;
                }
            }
        }

        AlignmentDirection::Bullish
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
