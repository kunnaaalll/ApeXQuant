//! Swing high/low detection using pivot logic

use crate::market_data::Candle;
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// A swing point (high or low)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwingPoint {
    /// Index in the candle series
    pub index: usize,
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// Price level
    pub price: Decimal,
    /// Type of swing
    pub swing_type: SwingType,
}

/// Type of swing point
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwingType {
    /// Swing high
    High,
    /// Swing low
    Low,
}

/// Collection of detected swings
#[derive(Debug, Clone, Default)]
pub struct Swings {
    /// Swing highs
    pub highs: Vec<SwingPoint>,
    /// Swing lows
    pub lows: Vec<SwingPoint>,
}

/// Detect swing highs and lows using N-bar pivot strategy
pub fn detect_swings(candles: &[Candle], pivot_bars: usize) -> Swings {
    let mut swings = Swings::default();

    if candles.len() < (pivot_bars * 2 + 1) {
        return swings;
    }

    for i in pivot_bars..candles.len() - pivot_bars {
        let candle = &candles[i];

        // Check for swing high
        let is_swing_high = (i - pivot_bars..i).all(|j| candles[j].high <= candle.high)
            && (i + 1..=i + pivot_bars).all(|j| candles[j].high <= candle.high);

        if is_swing_high {
            swings.highs.push(SwingPoint {
                index: i,
                timestamp: candle.timestamp,
                price: candle.high,
                swing_type: SwingType::High,
            });
        }

        // Check for swing low
        let is_swing_low = (i - pivot_bars..i).all(|j| candles[j].low >= candle.low)
            && (i + 1..=i + pivot_bars).all(|j| candles[j].low >= candle.low);

        if is_swing_low {
            swings.lows.push(SwingPoint {
                index: i,
                timestamp: candle.timestamp,
                price: candle.low,
                swing_type: SwingType::Low,
            });
        }
    }

    swings
}

/// Get the most recent N swing points
pub fn recent_swings(swings: &[SwingPoint], n: usize) -> Vec<SwingPoint> {
    swings.iter().rev().take(n).cloned().collect()
}

/// Get the most recent swing high
pub fn last_swing_high(swings: &[SwingPoint]) -> Option<&SwingPoint> {
    swings
        .iter()
        .filter(|s| s.swing_type == SwingType::High)
        .last()
}

/// Get the most recent swing low
pub fn last_swing_low(swings: &[SwingPoint]) -> Option<&SwingPoint> {
    swings
        .iter()
        .filter(|s| s.swing_type == SwingType::Low)
        .last()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn create_test_candles() -> Vec<Candle> {
        vec![
            // Index 0: Setup
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(100, 2),
                Decimal::new(105, 2),
                Decimal::new(98, 2),
                Decimal::new(103, 2),
                1000,
            ),
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(103, 2),
                Decimal::new(106, 2),
                Decimal::new(100, 2),
                Decimal::new(104, 2),
                1000,
            ),
            // Index 2: Swing high (1.10)
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(104, 2),
                Decimal::new(110, 2),
                Decimal::new(102, 2),
                Decimal::new(108, 2),
                1000,
            ),
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(108, 2),
                Decimal::new(109, 2),
                Decimal::new(103, 2),
                Decimal::new(105, 2),
                1000,
            ),
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(105, 2),
                Decimal::new(107, 2),
                Decimal::new(100, 2),
                Decimal::new(102, 2),
                1000,
            ),
            // Index 5: Swing low (0.95)
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(102, 2),
                Decimal::new(103, 2),
                Decimal::new(95, 2),
                Decimal::new(97, 2),
                1000,
            ),
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(97, 2),
                Decimal::new(99, 2),
                Decimal::new(96, 2),
                Decimal::new(98, 2),
                1000,
            ),
        ]
    }

    #[test]
    fn test_detect_swings() {
        let candles = create_test_candles();
        let swings = detect_swings(&candles, 1);

        // Should find 1 swing high at index 2
        assert_eq!(swings.highs.len(), 1);
        assert_eq!(swings.highs[0].index, 2);
        assert_eq!(swings.highs[0].price, Decimal::new(110, 2));

        // Should find 1 swing low at index 5
        assert_eq!(swings.lows.len(), 1);
        assert_eq!(swings.lows[0].index, 5);
        assert_eq!(swings.lows[0].price, Decimal::new(95, 2));
    }

    #[test]
    fn test_insufficient_data() {
        let candles = vec![Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(100, 2),
            Decimal::new(105, 2),
            Decimal::new(98, 2),
            Decimal::new(103, 2),
            1000,
        )];

        let swings = detect_swings(&candles, 3);
        assert!(swings.highs.is_empty());
        assert!(swings.lows.is_empty());
    }
}
