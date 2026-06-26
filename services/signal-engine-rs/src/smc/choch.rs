//! Change of Character (CHoCH) detection
//!
//! CHoCH indicates a shift in market structure, potentially signaling
//! a trend change or major correction.
use num_traits::ToPrimitive;

use crate::market_data::Candle;
use crate::structure::swings::{SwingPoint, SwingType};
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Change of Character pattern
#[derive(Debug, Clone)]
pub struct CHoCH {
    /// Direction of CHoCH
    pub direction: CHoCHDirection,
    /// Index of the break
    pub break_index: usize,
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// Price level of break
    pub level: Decimal,
    /// Break price
    pub break_price: Decimal,
    /// Strength (0.0 - 1.0)
    pub strength: f64,
    /// Age in bars
    pub age_bars: u32,
    /// Timeframe
    pub timeframe: String,
}

/// CHoCH direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CHoCHDirection {
    /// Bullish shift (broke above prior structure)
    Bullish,
    /// Bearish shift (broke below prior structure)
    Bearish,
}

/// Detect CHoCH patterns
pub fn detect_choch(
    candles: &[Candle],
    swings: &[SwingPoint],
    timeframe: &str,
) -> Vec<CHoCH> {
    let mut patterns = Vec::new();

    if candles.len() < 10 || swings.len() < 3 {
        return patterns;
    }

    // Group swings by type
    let highs: Vec<&SwingPoint> = swings.iter()
        .filter(|s| s.swing_type == SwingType::High)
        .collect();
    let lows: Vec<&SwingPoint> = swings.iter()
        .filter(|s| s.swing_type == SwingType::Low)
        .collect();

    // Look for bullish CHoCH: price breaks below a significant low then closes above it
    for (i, low) in lows.iter().enumerate() {
        if i == 0 {
            continue;
        }

        let prior_low = lows[i - 1];

        // Check for pattern: take out prior low, then close above it
        for j in (prior_low.index + 1)..=low.index.min(candles.len().saturating_sub(1)) {
            let candle = &candles[j];

            // Must take out the low (wick below), then close back above
            if candle.low < prior_low.price && candle.close > prior_low.price {
                // This is a bullish CHoCH
                let strength = calculate_choch_strength(
                    candle.close,
                    prior_low.price,
                    candle.low,
                    candles,
                    j,
                    true,
                );

                patterns.push(CHoCH {
                    direction: CHoCHDirection::Bullish,
                    break_index: j,
                    timestamp: candle.timestamp,
                    level: prior_low.price,
                    break_price: candle.close,
                    strength,
                    age_bars: (candles.len() - j) as u32,
                    timeframe: timeframe.to_string(),
                });
                break;
            }
        }
    }

    // Look for bearish CHoCH
    for (i, high) in highs.iter().enumerate() {
        if i == 0 {
            continue;
        }

        let prior_high = highs[i - 1];

        for j in (prior_high.index + 1)..=high.index.min(candles.len().saturating_sub(1)) {
            let candle = &candles[j];

            // Must take out the high (wick above), then close back below
            if candle.high > prior_high.price && candle.close < prior_high.price {
                let strength = calculate_choch_strength(
                    prior_high.price,
                    candle.close,
                    candle.high,
                    candles,
                    j,
                    false,
                );

                patterns.push(CHoCH {
                    direction: CHoCHDirection::Bearish,
                    break_index: j,
                    timestamp: candle.timestamp,
                    level: prior_high.price,
                    break_price: candle.close,
                    strength,
                    age_bars: (candles.len() - j) as u32,
                    timeframe: timeframe.to_string(),
                });
                break;
            }
        }
    }

    patterns
}

fn calculate_choch_strength(
    level: Decimal,
    close: Decimal,
    extreme: Decimal,
    candles: &[Candle],
    index: usize,
    bullish: bool,
) -> f64 {
    // Calculate wick size (rejection)
    let wick_size = if bullish {
        close - extreme
    } else {
        extreme - close
    };
    let body_size = (close - level).abs();

    // Strong CHoCH has significant rejection wick and good body
    let rejection_score = if wick_size > Decimal::ZERO {
        (body_size / wick_size.max(Decimal::new(1, 4)))
            .min(Decimal::from(2))
            .to_f64()
            .unwrap_or(0.5)
    } else {
        0.5
    };

    // Context from volatility
    let atr = calculate_atr_simple(candles, index);
    let move_multiple = if atr > Decimal::ZERO {
        body_size / atr
    } else {
        Decimal::ONE
    };

    let move_score = move_multiple.min(Decimal::from(2)).to_f64().unwrap_or(0.5);

    // Combined score
    (rejection_score * 0.4 + move_score * 0.6).clamp(0.2, 1.0)
}

fn calculate_atr_simple(candles: &[Candle], end_index: usize) -> Decimal {
    let start = end_index.saturating_sub(14).max(1);
    let end = end_index.min(candles.len());

    if end <= start {
        return Decimal::new(10, 4); // Default small value
    }

    let ranges: Vec<Decimal> = (start..end)
        .map(|i| candles[i].high - candles[i].low)
        .collect();

    let sum: Decimal = ranges.iter().sum();
    if ranges.is_empty() {
        Decimal::new(10, 4)
    } else {
        sum / Decimal::from(ranges.len() as i64)
    }
}

/// Check for recent CHoCH pattern
pub fn has_recent_choch(
    candles: &[Candle],
    swings: &[SwingPoint],
    lookback: usize,
    direction: CHoCHDirection,
) -> Option<CHoCH> {
    let patterns = detect_choch(candles, swings, "unknown");

    patterns.into_iter()
        .filter(|c| candles.len().saturating_sub(c.break_index) <= lookback)
        .filter(|c| c.direction == direction)
        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
}

/// Get CHoCH as trend bias indicator
pub fn get_structure_bias(ch_patterns: &[CHoCH]) -> Option<CHoCHDirection> {
    // Count recent CHoCH patterns
    let bullish_count = ch_patterns.iter()
        .filter(|c| c.direction == CHoCHDirection::Bullish)
        .count();
    let bearish_count = ch_patterns.iter()
        .filter(|c| c.direction == CHoCHDirection::Bearish)
        .count();

    if bullish_count > bearish_count * 2 {
        Some(CHoCHDirection::Bullish)
    } else if bearish_count > bullish_count * 2 {
        Some(CHoCHDirection::Bearish)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candle_with_wick(open: i64, high: i64, low: i64, close: i64) -> Candle {
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
    fn test_bullish_choch() {
        // Pattern: low gets run (wick below), then close above
        let candles = vec![
            create_candle_with_wick(10500, 10600, 10400, 10500),
            create_candle_with_wick(10500, 10550, 10250, 10400), // Wick below 10400
            create_candle_with_wick(10400, 10500, 10200, 10450), // Close back above, CHoCH
        ];

        let swings = vec![
            SwingPoint { index: 0, timestamp: OffsetDateTime::now_utc(), price: Decimal::new(10600, 2), swing_type: SwingType::High },
            SwingPoint { index: 1, timestamp: OffsetDateTime::now_utc(), price: Decimal::new(10250, 2), swing_type: SwingType::Low },
            SwingPoint { index: 2, timestamp: OffsetDateTime::now_utc(), price: Decimal::new(10500, 2), swing_type: SwingType::High },
        ];

        let choch = detect_choch(&candles, &swings, "M15");
        assert!(!choch.is_empty(), "Should detect CHoCH pattern");
    }
}
