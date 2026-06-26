//! Fair Value Gap (FVG) detection
//!
//! FVGs represent areas where price moved rapidly, leaving an unfilled gap
//! that often acts as a magnet for future price action.
use num_traits::ToPrimitive;

use crate::market_data::Candle;
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Fair Value Gap
#[derive(Debug, Clone)]
pub struct FairValueGap {
    /// Direction of FVG (bullish = gap below, bearish = gap above)
    pub direction: FVGDirection,
    /// Start index (first candle in formation)
    pub start_index: usize,
    /// End index (last candle in formation)
    pub end_index: usize,
    /// Timestamp of formation
    pub timestamp: OffsetDateTime,
    /// Top of gap zone (imbalance high)
    pub top: Decimal,
    /// Bottom of gap zone (imbalance low)
    pub bottom: Decimal,
    /// Gap size
    pub size: Decimal,
    /// Strength based on displacement
    pub strength: f64,
    /// Has FVG been filled/mitigated
    pub filled: bool,
    /// Fill index if filled
    pub fill_index: Option<usize>,
    /// Age in bars
    pub age_bars: u32,
    /// Timeframe
    pub timeframe: String,
}

/// FVG direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FVGDirection {
    /// Bullish FVG (gap below, price should bounce up from it)
    Bullish,
    /// Bearish FVG (gap above, price should reject from it)
    Bearish,
}

impl FairValueGap {
    /// Check if price is within the FVG zone
    pub fn contains(&self, price: Decimal) -> bool {
        price >= self.bottom && price <= self.top
    }

    /// Get fill percentage if partially filled
    pub fn fill_percentage(&self, current_low: Decimal, current_high: Decimal) -> f64 {
        let gap_size = self.size;
        if gap_size == Decimal::ZERO {
            return 0.0;
        }

        let overlap_bottom = current_low.max(self.bottom);
        let overlap_top = current_high.min(self.top);

        if overlap_top <= overlap_bottom {
            return 0.0;
        }

        let overlap = overlap_top - overlap_bottom;
        overlap.to_f64().unwrap_or(0.0) / gap_size.to_f64().unwrap_or(1.0)
    }

    /// Check if FVG is fresh (unfilled and reasonably new)
    pub fn is_fresh(&self, max_age: u32) -> bool {
        !self.filled && self.age_bars <= max_age
    }
}

/// Detect Fair Value Gaps from candle series
pub fn detect_fvgs(
    candles: &[Candle],
    timeframe: &str,
) -> Vec<FairValueGap> {
    let mut fvgs = Vec::new();

    if candles.len() < 3 {
        return fvgs;
    }

    // FVG detection: 3-candle pattern
    // Bullish: Candle 1 high < Candle 3 low (gap below)
    // Bearish: Candle 1 low > Candle 3 high (gap above)

    for i in 0..candles.len() - 2 {
        let c1 = &candles[i];
        let c2 = &candles[i + 1];
        let c3 = &candles[i + 2];

        // Bullish FVG: displacement up
        if c1.high < c3.low && c2.is_bullish() {
            let gap_size = c3.low - c1.high;
            let strength = calculate_fvg_strength(c1, c2, c3, true);

            fvgs.push(FairValueGap {
                direction: FVGDirection::Bullish,
                start_index: i,
                end_index: i + 2,
                timestamp: c2.timestamp,
                top: c3.low,
                bottom: c1.high,
                size: gap_size,
                strength,
                filled: false,
                fill_index: None,
                age_bars: (candles.len() - i) as u32,
                timeframe: timeframe.to_string(),
            });
        }

        // Bearish FVG: displacement down
        if c1.low > c3.high && c2.is_bearish() {
            let gap_size = c1.low - c3.high;
            let strength = calculate_fvg_strength(c1, c2, c3, false);

            fvgs.push(FairValueGap {
                direction: FVGDirection::Bearish,
                start_index: i,
                end_index: i + 2,
                timestamp: c2.timestamp,
                top: c1.low,
                bottom: c3.high,
                size: gap_size,
                strength,
                filled: false,
                fill_index: None,
                age_bars: (candles.len() - i) as u32,
                timeframe: timeframe.to_string(),
            });
        }
    }

    // Mark filled FVGs
    mark_filled(&mut fvgs, candles);

    fvgs
}

fn calculate_fvg_strength(
    c1: &Candle,
    c2: &Candle,
    c3: &Candle,
    bullish: bool,
) -> f64 {
    let mut score = 0.0;

    // Factor 1: Body size of displacement candle (c2)
    let c2_body = c2.body_size();
    let avg_range = ((c1.high - c1.low) + (c3.high - c3.low)) / Decimal::from(2);

    if avg_range > Decimal::ZERO {
        let ratio = c2_body / avg_range;
        score += ratio.min(Decimal::from(3)).to_f64().unwrap_or(0.0) * 0.3;
    }

    // Factor 2: Wick structure (strong FVGs have small wicks on displacement)
    let c2_range = c2.range();
    let wick_ratio = if c2_range > Decimal::ZERO {
        (c2_range - c2_body) / c2_range
    } else {
        Decimal::ZERO
    };

    score += (Decimal::ONE - wick_ratio).to_f64().unwrap_or(0.0) * 0.3;

    // Factor 3: Gap size relative to recent volatility
    let gap_size = if bullish {
        c3.low - c1.high
    } else {
        c1.low - c3.high
    };

    if avg_range > Decimal::ZERO {
        let gap_ratio = gap_size / avg_range;
        score += gap_ratio.min(Decimal::from(2)).to_f64().unwrap_or(0.0) * 0.4;
    }

    score.min(1.0)
}

fn mark_filled(fvgs: &mut [FairValueGap], candles: &[Candle]) {
    for fvg in fvgs.iter_mut() {
        for i in (fvg.end_index + 1)..candles.len() {
            let candle = &candles[i];

            match fvg.direction {
                FVGDirection::Bullish => {
                    // Bullish FVG filled when price trades down through it
                    if candle.low <= fvg.bottom {
                        fvg.filled = true;
                        fvg.fill_index = Some(i);
                        break;
                    }
                }
                FVGDirection::Bearish => {
                    // Bearish FVG filled when price trades up through it
                    if candle.high >= fvg.top {
                        fvg.filled = true;
                        fvg.fill_index = Some(i);
                        break;
                    }
                }
            }
        }
    }
}

/// Find fresh FVGs near current price
pub fn find_relevant_fvgs(
    fvgs: &[FairValueGap],
    current_price: Decimal,
    max_distance_multiple: f64,
) -> Vec<&FairValueGap> {
    let avg_fvg_size = fvgs.iter()
        .map(|f| f.size.to_f64().unwrap_or(0.0))
        .sum::<f64>() / fvgs.len().max(1) as f64;

    fvgs.iter()
        .filter(|f| f.is_fresh(50))
        .filter(|f| {
            let distance = match f.direction {
                FVGDirection::Bullish => (f.bottom - current_price).abs(),
                FVGDirection::Bearish => (f.top - current_price).abs(),
            };
            let distance_f64 = distance.to_f64().unwrap_or(f64::MAX);
            distance_f64 <= avg_fvg_size * max_distance_multiple
        })
        .collect()
}

/// Get nearest bullish and bearish FVGs
pub fn get_nearest_fvgs(
    fvgs: &[FairValueGap],
    current_price: Decimal,
) -> (Option<&FairValueGap>, Option<&FairValueGap>) {
    let fresh = fvgs.iter().filter(|f| f.is_fresh(50));

    let bullish = fresh.clone()
        .filter(|f| f.direction == FVGDirection::Bullish)
        .min_by(|a, b| {
            let da = (a.bottom - current_price).abs();
            let db = (b.bottom - current_price).abs();
            da.partial_cmp(&db).unwrap()
        });

    let bearish = fresh
        .filter(|f| f.direction == FVGDirection::Bearish)
        .min_by(|a, b| {
            let da = (a.top - current_price).abs();
            let db = (b.top - current_price).abs();
            da.partial_cmp(&db).unwrap()
        });

    (bullish, bearish)
}

/// FVG analysis result
#[derive(Debug, Clone)]
pub struct FVGAnalysis {
    /// Nearest bullish FVG
    pub nearest_bullish: Option<FairValueGap>,
    /// Nearest bearish FVG
    pub nearest_bearish: Option<FairValueGap>,
    /// Total fresh FVG count
    pub fresh_count: usize,
    /// Bullish bias from FVGs
    pub bullish_bias: f64,
}

/// Analyze FVG landscape
pub fn analyze_fvgs(fvgs: &[FairValueGap], current_price: Decimal) -> FVGAnalysis {
    let fresh: Vec<_> = fvgs.iter().filter(|f| f.is_fresh(50)).cloned().collect();
    let (bullish, bearish) = get_nearest_fvgs(&fresh, current_price);

    let bullish_count = fresh.iter()
        .filter(|f| f.direction == FVGDirection::Bullish)
        .count() as f64;
    let bearish_count = fresh.iter()
        .filter(|f| f.direction == FVGDirection::Bearish)
        .count() as f64;

    let total = bullish_count + bearish_count;
    let bullish_bias = if total > 0.0 {
        (bullish_count - bearish_count) / total
    } else {
        0.0
    };

    FVGAnalysis {
        nearest_bullish: bullish.cloned(),
        nearest_bearish: bearish.cloned(),
        fresh_count: fresh.len(),
        bullish_bias,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candle(open: i64, high: i64, low: i64, close: i64) -> Candle {
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
    fn test_bullish_fvg() {
        let candles = vec![
            create_candle(10000, 10100, 9900, 10050),  // Candle 1
            create_candle(10050, 10500, 10000, 10400), // Displacement up
            create_candle(10400, 10600, 10450, 10550), // Candle 3, low above c1 high
        ];

        let fvgs = detect_fvgs(&candles, "M15");

        assert!(!fvgs.is_empty(), "Should detect bullish FVG");
        assert_eq!(fvgs[0].direction, FVGDirection::Bullish);
        assert!(fvgs[0].top > fvgs[0].bottom);
    }

    #[test]
    fn test_fvg_not_filled() {
        let candles = vec![
            create_candle(10000, 10100, 9900, 10050),
            create_candle(10050, 10500, 10000, 10400),
            create_candle(10400, 10600, 10450, 10550),
            create_candle(10550, 10700, 10500, 10600),
        ];

        let fvgs = detect_fvgs(&candles, "M15");

        assert!(!fvgs.is_empty());
        assert!(!fvgs[0].filled, "FVG should not be filled");
    }
}
