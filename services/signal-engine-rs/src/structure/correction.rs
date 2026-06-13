//! Correction (pullback) detection

use crate::market_data::Candle;
use crate::structure::impulse::{ImpulseDirection, ImpulseWave};
use rust_decimal::Decimal;

/// Correction characteristics
#[derive(Debug, Clone)]
pub struct Correction {
    /// Start index
    pub start_index: usize,
    /// End index (None if ongoing)
    pub end_index: Option<usize>,
    /// Start price
    pub start_price: Decimal,
    /// End price (current if ongoing)
    pub end_price: Decimal,
    /// Direction of correction (opposite to trend)
    pub direction: CorrectionDirection,
    /// Depth as percentage of prior impulse
    pub depth_percent: f64,
    /// Whether correction is complete
    pub complete: bool,
}

/// Correction direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CorrectionDirection {
    /// Downward correction (in uptrend)
    Down,
    /// Upward correction (in downtrend)
    Up,
}

impl Correction {
    /// Calculate price change
    pub fn price_change(&self) -> Decimal {
        (self.end_price - self.start_price).abs()
    }

    /// Check if correction is shallow (<38.2%)
    pub fn is_shallow(&self) -> bool {
        self.depth_percent < 38.2
    }

    /// Check if correction is deep (>61.8%)
    pub fn is_deep(&self) -> bool {
        self.depth_percent > 61.8
    }
}

/// Detect correction following an impulse wave
pub fn detect_correction(
    candles: &[Candle],
    prior_impulse: &ImpulseWave,
) -> Option<Correction> {
    let start_idx = prior_impulse.end_index;

    if start_idx >= candles.len() - 1 {
        return None;
    }

    let start_price = candles[start_idx].close;
    let impulse_size = prior_impulse.price_change();

    // Correction direction is opposite to impulse
    let direction = match prior_impulse.direction {
        ImpulseDirection::Up => CorrectionDirection::Down,
        ImpulseDirection::Down => CorrectionDirection::Up,
    };

    // Look for movement in correction direction
    let mut current_idx = start_idx;
    let mut current_price = start_price;
    let mut max_retracement = Decimal::ZERO;

    for i in (start_idx + 1)..candles.len() {
        let candle = &candles[i];

        match direction {
            CorrectionDirection::Down => {
                // Looking for lower lows
                if candle.low < current_price {
                    current_price = candle.low;
                    current_idx = i;

                    let retracement = (start_price - current_price).abs();
                    if retracement > max_retracement {
                        max_retracement = retracement;
                    }
                } else if candle.close > start_price {
                    // Got a close above start = correction over
                    break;
                }
            }
            CorrectionDirection::Up => {
                // Looking for higher highs
                if candle.high > current_price {
                    current_price = candle.high;
                    current_idx = i;

                    let retracement = (current_price - start_price).abs();
                    if retracement > max_retracement {
                        max_retracement = retracement;
                    }
                } else if candle.close < start_price {
                    // Got a close below start = correction over
                    break;
                }
            }
        }
    }

    // Calculate depth percentage
    let depth_percent = if impulse_size > Decimal::ZERO {
        let retracement_f64: f64 = max_retracement.to_string().parse().unwrap_or(0.0);
        let impulse_f64: f64 = impulse_size.to_string().parse().unwrap_or(1.0);
        (retracement_f64 / impulse_f64) * 100.0
    } else {
        0.0
    };

    // Need minimum correction to be valid
    if depth_percent < 10.0 {
        return None;
    }

    Some(Correction {
        start_index: start_idx,
        end_index: if current_idx > start_idx {
            Some(current_idx)
        } else {
            None
        },
        start_price,
        end_price: current_price,
        direction,
        depth_percent,
        complete: current_idx > start_idx,
    })
}

/// Calculate Fibonacci retracement levels
pub fn fibonacci_levels(impulse_start: Decimal, impulse_end: Decimal) -> FibonacciLevels {
    let range = (impulse_end - impulse_start).abs();

    FibonacciLevels {
        level_0: if impulse_end > impulse_start {
            impulse_end
        } else {
            impulse_start
        },
        level_236: calculate_retracement(impulse_start, impulse_end, range, 0.236),
        level_382: calculate_retracement(impulse_start, impulse_end, range, 0.382),
        level_500: calculate_retracement(impulse_start, impulse_end, range, 0.5),
        level_618: calculate_retracement(impulse_start, impulse_end, range, 0.618),
        level_786: calculate_retracement(impulse_start, impulse_end, range, 0.786),
        level_1000: if impulse_end > impulse_start {
            impulse_start
        } else {
            impulse_end
        },
    }
}

fn calculate_retracement(start: Decimal, end: Decimal, range: Decimal, ratio: f64) -> Decimal {
    if end > start {
        // Uptrend - retracement down
        end - (range * Decimal::from_f64_retain(ratio).unwrap_or_default())
    } else {
        // Downtrend - retracement up
        end + (range * Decimal::from_f64_retain(ratio).unwrap_or_default())
    }
}

/// Fibonacci retracement levels
#[derive(Debug, Clone)]
pub struct FibonacciLevels {
    /// 0% level (impulse end)
    pub level_0: Decimal,
    /// 23.6% retracement
    pub level_236: Decimal,
    /// 38.2% retracement
    pub level_382: Decimal,
    /// 50% retracement
    pub level_500: Decimal,
    /// 61.8% retracement
    pub level_618: Decimal,
    /// 78.6% retracement
    pub level_786: Decimal,
    /// 100% retracement (impulse start)
    pub level_1000: Decimal,
}

impl FibonacciLevels {
    /// Get nearest level to a price
    pub fn nearest_level(&self, price: Decimal) -> (Decimal, &'static str) {
        let levels = [
            (self.level_0, "0%"),
            (self.level_236, "23.6%"),
            (self.level_382, "38.2%"),
            (self.level_500, "50%"),
            (self.level_618, "61.8%"),
            (self.level_786, "78.6%"),
            (self.level_1000, "100%"),
        ];

        levels
            .iter()
            .min_by_key(|(level, _)| (level - price).abs())
            .map(|(l, name)| (*l, *name))
            .unwrap_or((self.level_500, "50%"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::impulse::ImpulseWave;
    use time::OffsetDateTime;

    fn create_impulse_wave(start: i64, end: i64, up: bool) -> ImpulseWave {
        ImpulseWave {
            start_index: 0,
            end_index: 10,
            start_price: Decimal::new(start, 2),
            end_price: Decimal::new(end, 2),
            direction: if up {
                ImpulseDirection::Up
            } else {
                ImpulseDirection::Down
            },
            strength: 2.5,
            bars: 10,
        }
    }

    #[test]
    fn test_fibonacci_levels_uptrend() {
        let levels = fibonacci_levels(
            Decimal::new(10000, 2), // 100.00
            Decimal::new(11000, 2), // 110.00
        );

        assert_eq!(levels.level_0, Decimal::new(11000, 2));
        assert_eq!(levels.level_1000, Decimal::new(10000, 2));

        // 61.8% should be around 103.82
        assert!(levels.level_618 > Decimal::new(10380, 2));
        assert!(levels.level_618 < Decimal::new(10385, 2));
    }

    #[test]
    fn test_fibonacci_levels_downtrend() {
        let levels = fibonacci_levels(
            Decimal::new(11000, 2), // 110.00
            Decimal::new(10000, 2), // 100.00
        );

        assert_eq!(levels.level_0, Decimal::new(11000, 2));
        assert_eq!(levels.level_1000, Decimal::new(10000, 2));

        // 61.8% should be around 106.18
        assert!(levels.level_618 > Decimal::new(10615, 2));
        assert!(levels.level_618 < Decimal::new(10620, 2));
    }
}
