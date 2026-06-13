//! Range detection and analysis

use crate::structure::swings::SwingPoint;
use rust_decimal::Decimal;

/// Range structure
#[derive(Debug, Clone)]
pub struct RangeStructure {
    /// Upper boundary (resistance)
    pub upper: Decimal,
    /// Lower boundary (support)
    pub lower: Decimal,
    /// Number of touches on upper boundary
    pub upper_touches: u32,
    /// Number of touches on lower boundary
    pub lower_touches: u32,
    /// Bars within range
    pub bars_in_range: u32,
    /// Is range still valid
    pub valid: bool,
}

impl RangeStructure {
    /// Calculate range height
    pub fn height(&self) -> Decimal {
        self.upper - self.lower
    }

    /// Calculate midpoint
    pub fn midpoint(&self) -> Decimal {
        (self.upper + self.lower) / Decimal::from(2)
    }

    /// Check if price is within range
    pub fn contains(&self, price: Decimal) -> bool {
        price >= self.lower && price <= self.upper
    }

    /// Check if price is near upper boundary
    pub fn near_upper(&self, price: Decimal, tolerance: Decimal) -> bool {
        (self.upper - price).abs() <= tolerance
    }

    /// Check if price is near lower boundary
    pub fn near_lower(&self, price: Decimal, tolerance: Decimal) -> bool {
        (price - self.lower).abs() <= tolerance
    }
}

/// Detect if market is in a range
pub fn detect_range(swings_highs: &[SwingPoint], swing_lows: &[SwingPoint]) -> Option<RangeStructure> {
    // Need at least 2 touches on each side
    if swings_highs.len() < 2 || swing_lows.len() < 2 {
        return None;
    }

    // Get recent swing points
    let recent_highs: Vec<_> = swings_highs.iter().rev().take(4).collect();
    let recent_lows: Vec<_> = swing_lows.iter().rev().take(4).collect();

    // Check if highs are at similar levels (consolidation)
    let high_prices: Vec<_> = recent_highs.iter().map(|s| s.price).collect();
    let low_prices: Vec<_> = recent_lows.iter().map(|s| s.price).collect();

    let avg_high = high_prices.iter().sum::<Decimal>() / Decimal::from(high_prices.len() as i64);
    let avg_low = low_prices.iter().sum::<Decimal>() / Decimal::from(low_prices.len() as i64);

    // Check for similar highs (within 10 pips for forex)
    let high_variance = high_prices
        .iter()
        .map(|p| (*p - avg_high).abs())
        .max()
        .unwrap_or(Decimal::MAX);

    let low_variance = low_prices
        .iter()
        .map(|p| (*p - avg_low).abs())
        .max()
        .unwrap_or(Decimal::MAX);

    // Define tolerance based on range size (ATR-based would be better)
    let range_size = avg_high - avg_low;
    let tolerance = range_size / Decimal::from(10);

    if high_variance > tolerance || low_variance > tolerance {
        return None;
    }

    // Count touches
    let upper_touches = count_touches(swings_highs, avg_high, tolerance);
    let lower_touches = count_touches(swing_lows, avg_low, tolerance);

    // Need at least 2 touches on each side for valid range
    if upper_touches >= 2 && lower_touches >= 2 {
        Some(RangeStructure {
            upper: avg_high,
            lower: avg_low,
            upper_touches,
            lower_touches,
            bars_in_range: 0, // Would calculate from timestamps
            valid: true,
        })
    } else {
        None
    }
}

fn count_touches(swings: &[SwingPoint], level: Decimal, tolerance: Decimal) -> u32 {
    swings
        .iter()
        .filter(|s| (s.price - level).abs() <= tolerance)
        .count() as u32
}

/// Check for range breakout
pub fn check_breakout(
    range: &RangeStructure,
    current_price: Decimal,
    breakout_threshold: Decimal,
) -> Option<BreakoutDirection> {
    if current_price > range.upper + breakout_threshold {
        Some(BreakoutDirection::Up)
    } else if current_price < range.lower - breakout_threshold {
        Some(BreakoutDirection::Down)
    } else {
        None
    }
}

/// Breakout direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakoutDirection {
    /// Breakout to the upside
    Up,
    /// Breakout to the downside
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::swings::SwingType;
    use time::OffsetDateTime;

    fn create_swing_high(index: usize, price: i64) -> SwingPoint {
        SwingPoint {
            index,
            timestamp: OffsetDateTime::now_utc(),
            price: Decimal::new(price, 2),
            swing_type: SwingType::High,
        }
    }

    fn create_swing_low(index: usize, price: i64) -> SwingPoint {
        SwingPoint {
            index,
            timestamp: OffsetDateTime::now_utc(),
            price: Decimal::new(price, 2),
            swing_type: SwingType::Low,
        }
    }

    #[test]
    fn test_range_detection() {
        let highs = vec![
            create_swing_high(10, 11000), // 110.00
            create_swing_high(20, 11005), // 110.05
            create_swing_high(30, 11002), // 110.02
        ];

        let lows = vec![
            create_swing_low(5, 10800),   // 108.00
            create_swing_low(15, 10805),  // 108.05
            create_swing_low(25, 10802),  // 108.02
        ];

        let range = detect_range(&highs, &lows);
        assert!(range.is_some());

        let r = range.unwrap();
        assert!(r.upper > r.lower);
        assert_eq!(r.upper_touches, 3);
        assert_eq!(r.lower_touches, 3);
    }

    #[test]
    fn test_range_breakout_up() {
        let range = RangeStructure {
            upper: Decimal::new(11000, 2),
            lower: Decimal::new(10800, 2),
            upper_touches: 3,
            lower_touches: 3,
            bars_in_range: 50,
            valid: true,
        };

        let breakout = check_breakout(
            &range,
            Decimal::new(11050, 2), // 110.50
            Decimal::new(10, 2),    // 0.10 threshold
        );

        assert_eq!(breakout, Some(BreakoutDirection::Up));
    }
}
