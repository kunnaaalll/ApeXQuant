//! Trend detection based on market structure

use crate::market_data::Candle;
use crate::structure::swings::{SwingPoint, SwingType};

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    /// Uptrend (higher highs, higher lows)
    Up,
    /// Downtrend (lower highs, lower lows)
    Down,
    /// Sideways/ranging
    Sideways,
    /// No clear trend defined
    Undefined,
}

/// Trend structure
#[derive(Debug, Clone)]
pub struct Trend {
    /// Current trend direction
    pub direction: TrendDirection,
    /// Strength of trend (0.0 - 1.0)
    pub strength: f64,
    /// Number of bars in current trend
    pub bars_in_trend: u32,
    /// Last significant swing high
    pub last_high: Option<SwingPoint>,
    /// Last significant swing low
    pub last_low: Option<SwingPoint>,
}

/// Classify trend based on swing structure
pub fn classify_trend(
    candles: &[Candle],
    swing_highs: &[SwingPoint],
    swing_lows: &[SwingPoint],
) -> TrendDirection {
    if candles.len() < 2 {
        return TrendDirection::Undefined;
    }

    if swing_highs.len() >= 2 && swing_lows.len() >= 2 {
        let recent_highs: Vec<_> = swing_highs.iter().rev().take(3).collect();
        let recent_lows: Vec<_> = swing_lows.iter().rev().take(3).collect();

        let higher_highs =
            recent_highs.len() >= 2 && recent_highs.windows(2).all(|w| w[0].price > w[1].price);
        let higher_lows =
            recent_lows.len() >= 2 && recent_lows.windows(2).all(|w| w[0].price > w[1].price);

        if higher_highs && higher_lows {
            return TrendDirection::Up;
        }

        let lower_highs =
            recent_highs.len() >= 2 && recent_highs.windows(2).all(|w| w[0].price < w[1].price);
        let lower_lows =
            recent_lows.len() >= 2 && recent_lows.windows(2).all(|w| w[0].price < w[1].price);

        if lower_highs && lower_lows {
            return TrendDirection::Down;
        }
    }

    // Fallback: Price momentum direction (chronological: candles[0] is oldest, candles[len - 1] is newest)
    let len = candles.len();
    let oldest = candles[0].close;
    let newest = candles[len - 1].close;

    if newest > oldest {
        TrendDirection::Up
    } else if newest < oldest {
        TrendDirection::Down
    } else {
        TrendDirection::Sideways
    }
}

/// Calculate trend strength using ADX-like calculation
pub fn trend_strength(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 {
        return 0.0;
    }

    let recent = &candles[candles.len() - period..];

    // Count directional moves
    let up_moves: usize = recent
        .windows(2)
        .filter(|w| w[1].close > w[0].close)
        .count();

    let down_moves: usize = recent
        .windows(2)
        .filter(|w| w[1].close < w[0].close)
        .count();

    let total = recent.len() - 1;
    if total == 0 {
        return 0.0;
    }

    // Strength is max of up/down ratio
    let up_strength = up_moves as f64 / total as f64;
    let down_strength = down_moves as f64 / total as f64;

    up_strength.max(down_strength)
}

/// Detect if price is making a higher high pattern
pub fn is_higher_highs(swings: &[SwingPoint]) -> bool {
    if swings.len() < 2 {
        return false;
    }

    let highs: Vec<_> = swings
        .iter()
        .filter(|s| s.swing_type == SwingType::High)
        .collect();

    if highs.len() < 2 {
        return false;
    }

    highs.windows(2).all(|w| w[1].price > w[0].price)
}

/// Detect if price is making lower lows pattern
pub fn is_lower_lows(swings: &[SwingPoint]) -> bool {
    if swings.len() < 2 {
        return false;
    }

    let lows: Vec<_> = swings
        .iter()
        .filter(|s| s.swing_type == SwingType::Low)
        .collect();

    if lows.len() < 2 {
        return false;
    }

    lows.windows(2).all(|w| w[1].price < w[0].price)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use time::OffsetDateTime;

    #[test]
    fn test_uptrend_detection() {
        let candles = vec![
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                0
            );
            10
        ];

        let highs = vec![
            SwingPoint {
                index: 0,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(105, 2),
                swing_type: SwingType::High,
            },
            SwingPoint {
                index: 10,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(110, 2),
                swing_type: SwingType::High,
            },
        ];

        let lows = vec![
            SwingPoint {
                index: 5,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(100, 2),
                swing_type: SwingType::Low,
            },
            SwingPoint {
                index: 15,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(102, 2),
                swing_type: SwingType::Low,
            },
        ];

        let trend = classify_trend(&candles, &highs, &lows);
        assert_eq!(trend, TrendDirection::Up);
    }

    #[test]
    fn test_downtrend_detection() {
        let candles = vec![
            Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                0
            );
            10
        ];

        let highs = vec![
            SwingPoint {
                index: 0,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(110, 2),
                swing_type: SwingType::High,
            },
            SwingPoint {
                index: 10,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(105, 2),
                swing_type: SwingType::High,
            },
        ];

        let lows = vec![
            SwingPoint {
                index: 5,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(102, 2),
                swing_type: SwingType::Low,
            },
            SwingPoint {
                index: 15,
                timestamp: OffsetDateTime::now_utc(),
                price: Decimal::new(98, 2),
                swing_type: SwingType::Low,
            },
        ];

        let trend = classify_trend(&candles, &highs, &lows);
        assert_eq!(trend, TrendDirection::Down);
    }
}
