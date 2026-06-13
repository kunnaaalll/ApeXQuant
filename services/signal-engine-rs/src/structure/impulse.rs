//! Impulse wave detection

use crate::market_data::Candle;
use rust_decimal::Decimal;

/// Impulse wave characteristics
#[derive(Debug, Clone)]
pub struct ImpulseWave {
    /// Start index
    pub start_index: usize,
    /// End index
    pub end_index: usize,
    /// Start price
    pub start_price: Decimal,
    /// End price
    pub end_price: Decimal,
    /// Direction
    pub direction: ImpulseDirection,
    /// Strength based on ATR multiple
    pub strength: f64,
    /// Number of bars
    pub bars: u32,
}

/// Impulse direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImpulseDirection {
    /// Upward impulse
    Up,
    /// Downward impulse
    Down,
}

impl ImpulseWave {
    /// Calculate price change
    pub fn price_change(&self) -> Decimal {
        (self.end_price - self.start_price).abs()
    }

    /// Check if this is a strong impulse
    pub fn is_strong(&self) -> bool {
        self.strength >= 2.0
    }
}

/// Detect impulse waves in candle series
pub fn detect_impulses(
    candles: &[Candle],
    min_bars: usize,
    atr: Decimal,
    min_atr_multiple: f64,
) -> Vec<ImpulseWave> {
    let mut impulses = Vec::new();

    if candles.len() < min_bars + 1 {
        return impulses;
    }

    let atr_f64: f64 = atr.to_string().parse().unwrap_or(0.0);
    if atr_f64 == 0.0 {
        return impulses;
    }

    let mut i = 0;
    while i < candles.len() - min_bars {
        // Look for consecutive moves in same direction
        let mut up_count = 0usize;
        let mut down_count = 0usize;
        let start_price = candles[i].open;
        let mut end_idx = i;

        for j in i..candles.len() {
            if candles[j].is_bullish() {
                up_count += 1;
                down_count = 0;
            } else {
                down_count += 1;
                up_count = 0;
            }

            // Check if we have enough consecutive moves
            if up_count >= min_bars || down_count >= min_bars {
                end_idx = j;
                break;
            }

            // Reset if we see opposite direction
            if up_count > 0 && candles[j].is_bearish() {
                break;
            }
            if down_count > 0 && candles[j].is_bullish() {
                break;
            }
        }

        // Calculate total bars and price change
        let bars = end_idx - i + 1;
        if bars >= min_bars {
            let end_price = candles[end_idx].close;
            let price_change = (end_price - start_price).abs();
            let change_f64: f64 = price_change.to_string().parse().unwrap_or(0.0);
            let strength = change_f64 / atr_f64;

            if strength >= min_atr_multiple {
                let direction = if end_price > start_price {
                    ImpulseDirection::Up
                } else {
                    ImpulseDirection::Down
                };

                impulses.push(ImpulseWave {
                    start_index: i,
                    end_index: end_idx,
                    start_price,
                    end_price,
                    direction,
                    strength,
                    bars: bars as u32,
                });
            }
        }

        i = end_idx.max(i + 1);
    }

    impulses
}

/// Check if current candles show displacement (strong impulse)
pub fn has_displacement(
    candles: &[Candle],
    lookback: usize,
    atr: Decimal,
    threshold_multiple: f64,
) -> Option<ImpulseDirection> {
    if candles.len() < lookback {
        return None;
    }

    let recent = &candles[candles.len() - lookback..];
    let start_price = recent[0].open;
    let end_price = recent[recent.len() - 1].close;

    let price_change = (end_price - start_price).abs();
    let atr_f64: f64 = atr.to_string().parse().unwrap_or(0.0);
    let change_f64: f64 = price_change.to_string().parse().unwrap_or(0.0);

    if atr_f64 == 0.0 {
        return None;
    }

    let multiple = change_f64 / atr_f64;

    if multiple >= threshold_multiple {
        Some(if end_price > start_price {
            ImpulseDirection::Up
        } else {
            ImpulseDirection::Down
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_bullish_candle(open: i64, close: i64) -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(open, 2),
            Decimal::new(close + 10, 2),
            Decimal::new(open - 5, 2),
            Decimal::new(close, 2),
            1000,
        )
    }

    #[test]
    fn test_displacement_detection() {
        let candles = vec![
            create_bullish_candle(1000, 1005),
            create_bullish_candle(1005, 1015),
            create_bullish_candle(1015, 1025),
        ];

        let atr = Decimal::new(50, 2); // 0.50
        let displacement = has_displacement(&candles, 3, atr, 1.5);

        assert_eq!(displacement, Some(ImpulseDirection::Up));
    }

    #[test]
    fn test_no_displacement() {
        let candles = vec![
            create_bullish_candle(1000, 1002),
            create_bullish_candle(1002, 1003),
            create_bullish_candle(1003, 1004),
        ];

        let atr = Decimal::new(50, 2);
        let displacement = has_displacement(&candles, 3, atr, 2.0);

        assert_eq!(displacement, None);
    }
}
