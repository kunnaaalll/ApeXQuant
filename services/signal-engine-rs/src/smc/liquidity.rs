//! Liquidity sweep detection
//!
//! Liquidity sweeps occur when price briefly violates a significant level
//! (taking out stops) before reversing sharply.
use num_traits::ToPrimitive;

use crate::market_data::Candle;
use crate::structure::swings::{SwingPoint, SwingType};
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Liquidity sweep pattern
#[derive(Debug, Clone)]
pub struct LiquiditySweep {
    /// Sweep direction (where liquidity was taken)
    pub direction: SweepDirection,
    /// Level that was swept
    pub level: Decimal,
    /// Index of sweep candle
    pub sweep_index: usize,
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// Price that swept the level (wick extreme)
    pub sweep_price: Decimal,
    /// Reversal close price
    pub reversal_close: Decimal,
    /// Strength (0.0 - 1.0)
    pub strength: f64,
    /// Type of liquidity taken
    pub liquidity_type: LiquidityType,
    /// Was this an equal highs/lows sweep
    pub equal_levels: bool,
    /// Timeframe
    pub timeframe: String,
}

/// Sweep direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SweepDirection {
    /// Swept highs (induced longs, then reversed down)
    High,
    /// Swept lows (induced shorts, then reversed up)
    Low,
}

/// Type of liquidity
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiquidityType {
    /// Swing point liquidity
    Swing,
    /// Equal highs/lows liquidity
    EqualLevels,
    /// Previous day/week high/low
    Session,
    /// Breakout liquidity from consolidation
    Breakout,
}

/// Detect liquidity sweeps from price action
pub fn detect_sweeps(
    candles: &[Candle],
    swings: &[SwingPoint],
    timeframe: &str,
) -> Vec<LiquiditySweep> {
    let mut sweeps = Vec::new();

    if candles.len() < 5 {
        return sweeps;
    }

    // Find swing-based liquidity levels
    let swing_highs: Vec<&SwingPoint> = swings
        .iter()
        .filter(|s| s.swing_type == SwingType::High)
        .collect();
    let swing_lows: Vec<&SwingPoint> = swings
        .iter()
        .filter(|s| s.swing_type == SwingType::Low)
        .collect();

    // Look for high sweeps
    for high in &swing_highs {
        let result = check_sweep_at_level(
            candles,
            *high,
            high.price,
            SweepDirection::High,
            LiquidityType::Swing,
            timeframe,
        );
        if let Some(sweep) = result {
            sweeps.push(sweep);
        }
    }

    // Look for low sweeps
    for low in &swing_lows {
        let result = check_sweep_at_level(
            candles,
            *low,
            low.price,
            SweepDirection::Low,
            LiquidityType::Swing,
            timeframe,
        );
        if let Some(sweep) = result {
            sweeps.push(sweep);
        }
    }

    // Check for equal highs/lows
    sweeps.extend(detect_equal_level_sweeps(
        candles,
        &swing_highs,
        &swing_lows,
        timeframe,
    ));

    sweeps
}

fn check_sweep_at_level(
    candles: &[Candle],
    swing: &SwingPoint,
    level: Decimal,
    direction: SweepDirection,
    liquidity_type: LiquidityType,
    timeframe: &str,
) -> Option<LiquiditySweep> {
    // Look for sweep in candles after the swing
    for i in (swing.index + 1)..candles.len() {
        let candle = &candles[i];
        let prev_candle = if i > 0 { Some(&candles[i - 1]) } else { None };

        match direction {
            SweepDirection::High => {
                // Sweep: wick above level, close below
                if candle.high > level {
                    let swept_distance = candle.high - level;
                    let reversal = level - candle.close;

                    // Need some reversal to confirm sweep
                    if reversal > Decimal::ZERO && candle.close < candle.open {
                        let strength =
                            calculate_sweep_strength(swept_distance, reversal, candles, i, true);

                        return Some(LiquiditySweep {
                            direction: SweepDirection::High,
                            level,
                            sweep_index: i,
                            timestamp: candle.timestamp,
                            sweep_price: candle.high,
                            reversal_close: candle.close,
                            strength,
                            liquidity_type,
                            equal_levels: false,
                            timeframe: timeframe.to_string(),
                        });
                    }
                }
            }
            SweepDirection::Low => {
                // Sweep: wick below level, close above
                if candle.low < level {
                    let swept_distance = level - candle.low;
                    let reversal = candle.close - level;

                    if reversal > Decimal::ZERO && candle.close > candle.open {
                        let strength =
                            calculate_sweep_strength(swept_distance, reversal, candles, i, false);

                        return Some(LiquiditySweep {
                            direction: SweepDirection::Low,
                            level,
                            sweep_index: i,
                            timestamp: candle.timestamp,
                            sweep_price: candle.low,
                            reversal_close: candle.close,
                            strength,
                            liquidity_type,
                            equal_levels: false,
                            timeframe: timeframe.to_string(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn detect_equal_level_sweeps(
    candles: &[Candle],
    highs: &[&SwingPoint],
    lows: &[&SwingPoint],
    timeframe: &str,
) -> Vec<LiquiditySweep> {
    let mut sweeps = Vec::new();
    let tolerance = Decimal::new(5, 4); // 0.0005 for forex

    // Find equal highs
    for i in 0..highs.len() {
        for j in (i + 1)..highs.len() {
            let diff = (highs[i].price - highs[j].price).abs();

            if diff <= tolerance {
                // Found equal highs - check for sweep
                if let Some(sweep) = check_sweep_at_level(
                    candles,
                    highs[j],
                    highs[j].price,
                    SweepDirection::High,
                    LiquidityType::EqualLevels,
                    timeframe,
                ) {
                    sweeps.push(LiquiditySweep {
                        equal_levels: true,
                        ..sweep
                    });
                }
            }
        }
    }

    // Find equal lows
    for i in 0..lows.len() {
        for j in (i + 1)..lows.len() {
            let diff = (lows[i].price - lows[j].price).abs();

            if diff <= tolerance {
                if let Some(sweep) = check_sweep_at_level(
                    candles,
                    lows[j],
                    lows[j].price,
                    SweepDirection::Low,
                    LiquidityType::EqualLevels,
                    timeframe,
                ) {
                    sweeps.push(LiquiditySweep {
                        equal_levels: true,
                        ..sweep
                    });
                }
            }
        }
    }

    sweeps
}

fn calculate_sweep_strength(
    swept_distance: Decimal,
    reversal: Decimal,
    candles: &[Candle],
    index: usize,
    was_high: bool,
) -> f64 {
    let mut score = 0.0;

    // Factor 1: Rejection quality (reversal vs sweep size)
    if swept_distance > Decimal::ZERO {
        let rejection_ratio = reversal / swept_distance;
        score += rejection_ratio
            .min(Decimal::from(3))
            .to_f64()
            .unwrap_or(0.0)
            * 0.35;
    }

    // Factor 2: Body confirmation
    let candle = &candles[index];
    let body = candle.body_size();
    let range = candle.range();

    if range > Decimal::ZERO {
        // Strong if body is in direction of reversal
        let body_direction = if was_high {
            candle.close < candle.open
        } else {
            candle.close > candle.open
        };

        if body_direction {
            score += (body / range).to_f64().unwrap_or(0.0) * 0.35;
        }
    }

    // Factor 3: Context (volatile environment = less reliable)
    let volatility = calculate_recent_volatility(candles, index);
    score += (1.0 - volatility.min(1.0)) * 0.3;

    score.min(1.0)
}

fn calculate_recent_volatility(candles: &[Candle], end_index: usize) -> f64 {
    let start = end_index.saturating_sub(14).max(1);
    let end = end_index.min(candles.len());

    if end <= start {
        return 0.5;
    }

    let ranges: Vec<f64> = (start..end)
        .map(|i| (candles[i].high - candles[i].low).to_f64().unwrap_or(0.0))
        .filter(|&r| r > 0.0)
        .collect();

    if ranges.is_empty() {
        return 0.5;
    }

    let mean = ranges.iter().sum::<f64>() / ranges.len() as f64;
    let variance = ranges.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / ranges.len() as f64;

    let cv = variance.sqrt() / mean; // Coefficient of variation
    cv.min(1.0)
}

/// Check for recent liquidity sweep
pub fn has_recent_sweep(
    candles: &[Candle],
    swings: &[SwingPoint],
    lookback: usize,
    direction: SweepDirection,
) -> Option<LiquiditySweep> {
    let sweeps = detect_sweeps(candles, swings, "unknown");

    sweeps
        .into_iter()
        .filter(|s| candles.len().saturating_sub(s.sweep_index) <= lookback)
        .filter(|s| s.direction == direction)
        .max_by(|a, b| {
            a.strength
                .partial_cmp(&b.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Get sweep bias from recent sweeps
pub fn get_sweep_bias(
    sweeps: &[LiquiditySweep],
    lookback: usize,
    total_candles: usize,
) -> Option<SweepDirection> {
    let recent: Vec<_> = sweeps
        .iter()
        .filter(|s| total_candles.saturating_sub(s.sweep_index) <= lookback)
        .collect();

    let high_sweeps = recent
        .iter()
        .filter(|s| s.direction == SweepDirection::High)
        .count();
    let low_sweeps = recent
        .iter()
        .filter(|s| s.direction == SweepDirection::Low)
        .count();

    if high_sweeps > low_sweeps * 2 && high_sweeps >= 2 {
        Some(SweepDirection::High) // Swept highs = likely bearish after
    } else if low_sweeps > high_sweeps * 2 && low_sweeps >= 2 {
        Some(SweepDirection::Low) // Swept lows = likely bullish after
    } else {
        None
    }
}

/// Liquidity analysis summary
#[derive(Debug, Clone)]
pub struct LiquidityAnalysis {
    /// Recent sweep count by direction
    pub recent_high_sweeps: usize,
    pub recent_low_sweeps: usize,
    /// Strongest recent sweep
    pub strongest_sweep: Option<LiquiditySweep>,
    /// Bias from sweep activity
    pub bias: Option<SweepDirection>,
}

/// Analyze liquidity conditions
pub fn analyze_liquidity(candles: &[Candle], swings: &[SwingPoint]) -> LiquidityAnalysis {
    let sweeps = detect_sweeps(candles, swings, "unknown");
    let lookback = 20;

    let recent: Vec<_> = sweeps
        .iter()
        .filter(|s| candles.len().saturating_sub(s.sweep_index) <= lookback)
        .collect();

    let high_sweeps = recent
        .iter()
        .filter(|s| s.direction == SweepDirection::High)
        .count();
    let low_sweeps = recent
        .iter()
        .filter(|s| s.direction == SweepDirection::Low)
        .count();

    let strongest = recent.iter().max_by(|a, b| {
        a.strength
            .partial_cmp(&b.strength)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let bias = get_sweep_bias(&sweeps, lookback, candles.len());

    LiquidityAnalysis {
        recent_high_sweeps: high_sweeps,
        recent_low_sweeps: low_sweeps,
        strongest_sweep: strongest.cloned().cloned(),
        bias,
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
    fn test_high_sweep_detection() {
        let mut candles = vec![create_candle_with_wick(10000, 10100, 9900, 10050); 2];

        candles.extend(vec![
            create_candle_with_wick(10000, 10100, 9900, 10050),
            create_candle_with_wick(10050, 10200, 10000, 10200), // Sets high
            create_candle_with_wick(10200, 10300, 10100, 10150), // Sweeps 102 with wick 103, closes lower
        ]);

        // Create a swing high at index 3 (2 + 1) with price 102.00
        let swings = vec![SwingPoint {
            index: 3,
            timestamp: OffsetDateTime::now_utc(),
            price: Decimal::new(10200, 2),
            swing_type: SwingType::High,
        }];

        let sweeps = detect_sweeps(&candles, &swings, "M15");

        assert!(!sweeps.is_empty(), "Should detect liquidity sweep");
        assert_eq!(sweeps[0].direction, SweepDirection::High);
    }
}
