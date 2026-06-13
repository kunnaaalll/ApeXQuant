//! Break of Structure (BOS) detection
//!
//! BOS occurs when price breaks a significant swing high/low,
//! indicating continuation of the current trend.

use crate::market_data::Candle;
use crate::structure::swings::{SwingPoint, SwingType};
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Break of Structure pattern
#[derive(Debug, Clone)]
pub struct BOS {
    /// Direction of break
    pub direction: BOSDirection,
    /// Index of the break candle
    pub break_index: usize,
    /// Timestamp of break
    pub timestamp: OffsetDateTime,
    /// Price level of broken swing
    pub level: Decimal,
    /// Price at break
    pub break_price: Decimal,
    /// Strength (0.0 - 1.0)
    pub strength: f64,
    /// Prior swing that was broken
    pub prior_swing: SwingPoint,
    /// Timeframe
    pub timeframe: String,
}

/// BOS direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BOSDirection {
    /// Bullish break (broke above swing high)
    Bullish,
    /// Bearish break (broke below swing low)
    Bearish,
}

/// Detect BOS patterns from candles and swings
pub fn detect_bos(
    candles: &[Candle],
    swings: &[SwingPoint],
    timeframe: &str,
) -> Vec<BOS> {
    if candles.len() < 5 || swings.len() < 2 {
        return Vec::new();
    }

    let mut bos_patterns = Vec::new();
    let swing_highs: Vec<&SwingPoint> = swings.iter()
        .filter(|s| s.swing_type == SwingType::High)
        .collect();
    let swing_lows: Vec<&SwingPoint> = swings.iter()
        .filter(|s| s.swing_type == SwingType::Low)
        .collect();

    // Look for bullish BOS (close above prior swing high)
    for window in swing_highs.windows(2) {
        let prior = window[0];
        let recent = window[1];

        if recent.index <= prior.index {
            continue;
        }

        // Check for break between these swings
        for i in (prior.index + 1)..recent.index.min(candles.len()) {
            let candle = &candles[i];

            // Bullish BOS: close above prior swing high
            if candle.close > prior.price && candle.index >= recent.index {
                let strength = calculate_bos_strength(candle.close, prior.price, candles, i);

                bos_patterns.push(BOS {
                    direction: BOSDirection::Bullish,
                    break_index: i,
                    timestamp: candle.timestamp,
                    level: prior.price,
                    break_price: candle.close,
                    strength,
                    prior_swing: *prior,
                    timeframe: timeframe.to_string(),
                });
                break;
            }
        }
    }

    // Look for bearish BOS
    for window in swing_lows.windows(2) {
        let prior = window[0];
        let recent = window[1];

        if recent.index <= prior.index {
            continue;
        }

        for i in (prior.index + 1)..recent.index.min(candles.len()) {
            let candle = &candles[i];

            // Bearish BOS: close below prior swing low
            if candle.close < prior.price && candle.index >= recent.index {
                let strength = calculate_bos_strength(prior.price, candle.close, candles, i);

                bos_patterns.push(BOS {
                    direction: BOSDirection::Bearish,
                    break_index: i,
                    timestamp: candle.timestamp,
                    level: prior.price,
                    break_price: candle.close,
                    strength,
                    prior_swing: *prior,
                    timeframe: timeframe.to_string(),
                });
                break;
            }
        }
    }

    bos_patterns
}

fn calculate_bos_strength(
    break_price: Decimal,
    level: Decimal,
    candles: &[Candle],
    index: usize,
) -> f64 {
    let move_size = (break_price - level).abs();

    // Get recent ATR for context
    let atr = calculate_atr(candles, index.saturating_sub(14), index);

    if atr == Decimal::ZERO {
        return 0.5;
    }

    let multiple = move_size / atr;

    // Higher strength for clean breaks (1+ ATR moves)
    match multiple {
        m if m >= Decimal::from(3) => 1.0,
        m if m >= Decimal::from(2) => 0.8,
        m if m >= Decimal::from(1) => 0.6,
        _ => 0.4,
    }
}

fn calculate_atr(candles: &[Candle], start: usize, end: usize) -> Decimal {
    let start = start.max(1);
    let end = end.min(candles.len());

    if end <= start {
        return Decimal::ZERO;
    }

    let mut sum = Decimal::ZERO;
    let mut count = 0usize;

    for i in start..end {
        let tr = (candles[i].high - candles[i].low)
            .max((candles[i].high - candles[i - 1].close).abs())
            .max((candles[i].low - candles[i - 1].close).abs());
        sum += tr;
        count += 1;
    }

    if count == 0 {
        Decimal::ZERO
    } else {
        sum / Decimal::from(count as i64)
    }
}

/// Check if recent price action contains a BOS
pub fn has_recent_bos(
    candles: &[Candle],
    swings: &[SwingPoint],
    lookback: usize,
    direction: BOSDirection,
) -> Option<BOS> {
    let patterns = detect_bos(candles, swings, "unknown");

    patterns.into_iter()
        .filter(|bos| candles.len().saturating_sub(bos.break_index) <= lookback)
        .filter(|bos| bos.direction == direction)
        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candle(high: i64, low: i64, close: i64) -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new((high + low) / 2, 2),
            Decimal::new(high, 2),
            Decimal::new(low, 2),
            Decimal::new(close, 2),
            1000,
        )
    }

    #[test]
    fn test_bullish_bos_detection() {
        let candles = vec![
            create_candle(10000, 9800, 9900), // Setup
            create_candle(10200, 10000, 10100),
            create_candle(10400, 10200, 10300),
            create_candle(10600, 10300, 10500),
            create_candle(10800, 10500, 10700), // Breaks swing high of 10400
        ];

        let swings = vec![
            SwingPoint { index: 0, timestamp: OffsetDateTime::now_utc(), price: Decimal::new(10000, 2), swing_type: SwingType::High },
            SwingPoint { index: 2, timestamp: OffsetDateTime::now_utc(), price: Decimal::new(10400, 2), swing_type: SwingType::High },
        ];

        let bos = detect_bos(&candles, &swings, "M15");

        // Should find bullish BOS
        assert!(!bos.is_empty());
        assert_eq!(bos[0].direction, BOSDirection::Bullish);
    }
}
