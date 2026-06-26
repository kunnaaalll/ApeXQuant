//! Mitigation detection
//!
//! Mitigation occurs when price returns to a previously respected level
//! (like an order block or FVG) and shows a reaction.
use num_traits::ToPrimitive;

use crate::market_data::Candle;
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Mitigation event
#[derive(Debug, Clone)]
pub struct Mitigation {
    /// Level that was mitigated
    pub level: Decimal,
    /// Type of level
    pub level_type: LevelType,
    /// Index of mitigation
    pub index: usize,
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// Direction of reaction
    pub reaction_direction: ReactionDirection,
    /// Strength of reaction
    pub strength: f64,
    /// Price at mitigation
    pub price: Decimal,
    /// Was it a full or partial mitigation
    pub mitigation_type: MitigationType,
}

/// Type of level that was mitigated
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelType {
    /// Order block
    OrderBlock,
    /// Fair value gap
    FairValueGap,
    /// Previous swing point
    SwingPoint,
    /// Breaker level
    Breaker,
    /// Session high/low
    SessionLevel,
}

/// Reaction direction after mitigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReactionDirection {
    /// Bounced up from level
    BounceUp,
    /// Rejected down from level
    RejectDown,
}

/// Type of mitigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MitigationType {
    /// Full level was tested
    Full,
    /// Partial test (50%+)
    Partial,
    /// Wick only
    Wick,
}

/// Check if price has mitigated a specific level
pub fn check_mitigation(
    candles: &[Candle],
    level_bottom: Decimal,
    level_top: Decimal,
    level_type: LevelType,
    lookback_start: usize,
) -> Option<Mitigation> {
    for i in lookback_start..candles.len() {
        let candle = &candles[i];

        // Check if price reached the level
        let reached_level = candle.low <= level_top && candle.high >= level_bottom;

        if reached_level {
            // Determine reaction
            let range = level_top - level_bottom;
            let penetration = if candle.low < level_bottom {
                level_bottom - candle.low
            } else {
                Decimal::ZERO
            };

            let mitigation_type = if penetration > range * Decimal::from_f64_retain(0.5).unwrap_or_default() {
                MitigationType::Full
            } else if penetration > Decimal::ZERO {
                MitigationType::Partial
            } else {
                MitigationType::Wick
            };

            // Determine direction based on close relative to level
            let mid = (level_top + level_bottom) / Decimal::from(2);
            let reaction_direction = if candle.close > mid {
                ReactionDirection::BounceUp
            } else {
                ReactionDirection::RejectDown
            };

            // Calculate strength
            let strength = calculate_mitigation_strength(
                candle,
                reaction_direction,
                level_bottom,
                level_top,
            );

            return Some(Mitigation {
                level: mid,
                level_type,
                index: i,
                timestamp: candle.timestamp,
                reaction_direction,
                strength,
                price: candle.close,
                mitigation_type,
            });
        }
    }

    None
}

fn calculate_mitigation_strength(
    candle: &Candle,
    direction: ReactionDirection,
    level_bottom: Decimal,
    level_top: Decimal,
) -> f64 {
    let mut score = 0.0;

    // Factor 1: Strong close in reaction direction
    let body = candle.body_size();
    let range = candle.range();

    if range > Decimal::ZERO {
        let body_ratio = body / range;
        score += body_ratio.to_f64().unwrap_or(0.0) * 0.3;
    }

    // Factor 2: Wick rejection
    match direction {
        ReactionDirection::BounceUp => {
            let lower_wick = candle.lower_wick();
            if lower_wick > Decimal::ZERO {
                let wick_ratio = lower_wick / range.max(Decimal::new(1, 4));
                score += wick_ratio.min(Decimal::ONE).to_f64().unwrap_or(0.0) * 0.3;
            }
        }
        ReactionDirection::RejectDown => {
            let upper_wick = candle.upper_wick();
            if upper_wick > Decimal::ZERO {
                let wick_ratio = upper_wick / range.max(Decimal::new(1, 4));
                score += wick_ratio.min(Decimal::ONE).to_f64().unwrap_or(0.0) * 0.3;
            }
        }
    }

    // Factor 3: Went into level then reversed (confirmation)
    let mid = (level_top + level_bottom) / Decimal::from(2);
    let went_into_level = candle.low <= level_top && candle.high >= level_bottom;

    if went_into_level {
        let close_direction = match direction {
            ReactionDirection::BounceUp => candle.close > mid,
            ReactionDirection::RejectDown => candle.close < mid,
        };

        if close_direction {
            score += 0.4;
        }
    }

    score.min(1.0)
}

/// Mitigation analysis for multiple levels
#[derive(Debug, Clone)]
pub struct MitigationAnalysis {
    /// Recent mitigations detected
    pub mitigations: Vec<Mitigation>,
    /// Bullish mitigation count
    pub bullish_count: usize,
    /// Bearish mitigation count
    pub bearish_count: usize,
    /// Strongest recent mitigation
    pub strongest: Option<Mitigation>,
    /// Bias from mitigations
    pub bias: Option<ReactionDirection>,
}

/// Analyze mitigation patterns
pub fn analyze_mitigations(
    candles: &[Candle],
    levels: &[(Decimal, Decimal, LevelType)],
    lookback: usize,
) -> MitigationAnalysis {
    let start = candles.len().saturating_sub(lookback);
    let mut mitigations = Vec::new();

    for (bottom, top, level_type) in levels {
        if let Some(mitigation) = check_mitigation(candles, *bottom, *top, *level_type, start) {
            mitigations.push(mitigation);
        }
    }

    // Sort by recency
    mitigations.sort_by(|a, b| b.index.cmp(&a.index));

    let bullish_count = mitigations.iter()
        .filter(|m| m.reaction_direction == ReactionDirection::BounceUp)
        .count();
    let bearish_count = mitigations.iter()
        .filter(|m| m.reaction_direction == ReactionDirection::RejectDown)
        .count();

    let strongest = mitigations.iter().max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap()).cloned();

    let bias = if bullish_count > bearish_count * 2 {
        Some(ReactionDirection::BounceUp)
    } else if bearish_count > bullish_count * 2 {
        Some(ReactionDirection::RejectDown)
    } else {
        None
    };

    MitigationAnalysis {
        mitigations,
        bullish_count,
        bearish_count,
        strongest,
        bias,
    }
}

/// Check if a specific level is currently being tested
pub fn is_level_being_tested(
    candles: &[Candle],
    bottom: Decimal,
    top: Decimal,
) -> bool {
    if let Some(last) = candles.last() {
        last.low <= top && last.high >= bottom
    } else {
        false
    }
}

/// Get distance to nearest untested level
pub fn distance_to_level(
    candles: &[Candle],
    level: Decimal,
) -> Option<(Decimal, bool)> {
    let last = candles.last()?;
    let current_price = last.close;

    let distance = (level - current_price).abs();
    let above = level > current_price;

    Some((distance, above))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candle_ohlc(open: i64, high: i64, low: i64, close: i64) -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(open, 2),
            Decimal::new(high, 2),
            Decimal::new(low, 2),
            Decimal::new(close, 2),
            1000,
        )
    }

    #[test]
    fn test_mitigation_detection() {
        let candles = vec![
            create_candle_ohlc(10500, 10600, 10400, 10500),
            create_candle_ohlc(10500, 10550, 10200, 10400), // Wicks into level 10200-10300, closes higher
            create_candle_ohlc(10400, 10500, 10350, 10450),
        ];

        let level_bottom = Decimal::new(10200, 2);
        let level_top = Decimal::new(10300, 2);

        let mitigation = check_mitigation(&candles, level_bottom, level_top, LevelType::OrderBlock, 0);

        assert!(mitigation.is_some());
        assert_eq!(mitigation.unwrap().reaction_direction, ReactionDirection::BounceUp);
    }
}
