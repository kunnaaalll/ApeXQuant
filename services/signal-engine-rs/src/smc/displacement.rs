//! Displacement detection
//!
//! Displacement is a strong, impulsive move indicating institutional participation
//! and often marking the start of a new trend or significant continuation.
use num_traits::ToPrimitive;

use crate::market_data::Candle;
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Displacement pattern
#[derive(Debug, Clone)]
pub struct Displacement {
    /// Direction of displacement
    pub direction: DisplacementDirection,
    /// Start index
    pub start_index: usize,
    /// End index
    pub end_index: usize,
    /// Start timestamp
    pub start_time: OffsetDateTime,
    /// End timestamp
    pub end_time: OffsetDateTime,
    /// Start price
    pub start_price: Decimal,
    /// End price
    pub end_price: Decimal,
    /// Number of bars
    pub bars: u32,
    /// Size of displacement
    pub size: Decimal,
    /// Strength relative to volatility
    pub strength: f64,
    /// ATR multiple achieved
    pub atr_multiple: f64,
    /// Was this a continuation or reversal
    pub displacement_type: DisplacementType,
    /// Timeframe
    pub timeframe: String,
}

/// Displacement direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplacementDirection {
    /// Upward displacement
    Up,
    /// Downward displacement
    Down,
}

/// Type of displacement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplacementType {
    /// Continuation of prior trend
    Continuation,
    /// Reversal of prior trend
    Reversal,
    /// Breakout from consolidation
    Breakout,
}

impl Displacement {
    /// Calculate price range of displacement
    pub fn price_range(&self) -> Decimal {
        (self.end_price - self.start_price).abs()
    }

    /// Check if this is a strong displacement
    pub fn is_strong(&self) -> bool {
        self.strength >= 0.7 && self.atr_multiple >= 2.0
    }

    /// Get midpoint of displacement
    pub fn midpoint(&self) -> Decimal {
        (self.start_price + self.end_price) / Decimal::from(2)
    }
}

/// Detect displacement moves from candle series
pub fn detect_displacements(
    candles: &[Candle],
    timeframe: &str,
    min_bars: usize,
    max_bars: usize,
    min_atr_multiple: f64,
) -> Vec<Displacement> {
    let mut displacements = Vec::new();

    if candles.len() < min_bars + 1 {
        return displacements;
    }

    // Calculate base volatility (ATR)
    let atr = calculate_atr(candles, 14);

    if atr == Decimal::ZERO {
        return displacements;
    }

    let mut i = min_bars;
    while i < candles.len() {
        // Look for consecutive moves in same direction with momentum
        let result = analyze_displacement_window(
            &candles[i.saturating_sub(max_bars)..=i],
            atr,
            i.saturating_sub(max_bars),
        );

        if let Some((direction, start_idx, strength, atr_mult)) = result {
            if atr_mult >= min_atr_multiple {
                let start_candle = &candles[start_idx];
                let end_candle = &candles[i];

                let displacement_type =
                    classify_displacement_type(&candles[..start_idx], direction, start_idx);

                displacements.push(Displacement {
                    direction,
                    start_index: start_idx,
                    end_index: i,
                    start_time: start_candle.timestamp,
                    end_time: end_candle.timestamp,
                    start_price: start_candle.open,
                    end_price: end_candle.close,
                    bars: (i - start_idx + 1) as u32,
                    size: (end_candle.close - start_candle.open).abs(),
                    strength,
                    atr_multiple: atr_mult,
                    displacement_type,
                    timeframe: timeframe.to_string(),
                });

                // Skip past this displacement
                i += min_bars;
                continue;
            }
        }

        i += 1;
    }

    displacements
}

fn analyze_displacement_window(
    candles: &[Candle],
    atr: Decimal,
    global_start: usize,
) -> Option<(DisplacementDirection, usize, f64, f64)> {
    if candles.len() < 3 {
        return None;
    }

    let start_price = candles[0].open;
    let end_price = candles[candles.len() - 1].close;
    let price_change = (end_price - start_price).abs();

    // Not enough magnitude
    if price_change < atr {
        return None;
    }

    // Count directional moves
    let mut up_bars = 0usize;
    let mut down_bars = 0usize;
    let mut max_shallow_pullback = Decimal::ZERO;

    for window in candles.windows(2) {
        if window[1].close > window[0].close {
            up_bars += 1;
        } else if window[1].close < window[0].close {
            down_bars += 1;
        }

        // Track pullbacks
        let pullback = (window[1].close - window[0].close).abs();
        max_shallow_pullback = max_shallow_pullback.max(pullback);
    }

    // Determine direction
    let direction = if up_bars > down_bars && end_price > start_price {
        DisplacementDirection::Up
    } else if down_bars > up_bars && end_price < start_price {
        DisplacementDirection::Down
    } else {
        return None;
    };

    // Calculate strength
    let bars = candles.len();
    let directional_bias = (up_bars.max(down_bars) as f64) / (bars as f64);
    let magnitude_score = (price_change / atr)
        .min(Decimal::from(5))
        .to_f64()
        .unwrap_or(1.0)
        / 5.0;
    let consistency_score = if max_shallow_pullback > Decimal::ZERO {
        1.0 - (max_shallow_pullback / price_change)
            .min(Decimal::ONE)
            .to_f64()
            .unwrap_or(0.0)
    } else {
        1.0
    };

    let strength = directional_bias * 0.4 + magnitude_score * 0.4 + consistency_score * 0.2;
    let atr_multiple = (price_change / atr).to_f64().unwrap_or(0.0);

    Some((direction, global_start, strength, atr_multiple))
}

fn classify_displacement_type(
    prior_candles: &[Candle],
    direction: DisplacementDirection,
    _start_idx: usize,
) -> DisplacementType {
    if prior_candles.len() < 10 {
        return DisplacementType::Continuation;
    }

    let recent = &prior_candles[prior_candles.len() - 10..];
    let start_price = recent[0].open;
    let end_price = recent[recent.len() - 1].close;
    let prior_direction = if end_price > start_price {
        DisplacementDirection::Up
    } else {
        DisplacementDirection::Down
    };

    // Check for consolidation (low volatility)
    let range = recent
        .iter()
        .map(|c| c.high)
        .fold(Decimal::MIN, |a, b| a.max(b))
        - recent
            .iter()
            .map(|c| c.low)
            .fold(Decimal::MAX, |a, b| a.min(b));
    let avg_range: Decimal =
        recent.iter().map(|c| c.range()).sum::<Decimal>() / Decimal::from(recent.len() as i64);

    let is_consolidation = range < avg_range * Decimal::from(5);

    if is_consolidation {
        DisplacementType::Breakout
    } else if prior_direction != direction {
        DisplacementType::Reversal
    } else {
        DisplacementType::Continuation
    }
}

fn calculate_atr(candles: &[Candle], period: usize) -> Decimal {
    if candles.len() < period + 1 {
        return Decimal::new(10, 4); // Default small value
    }

    let mut sum = Decimal::ZERO;
    let start = candles.len() - period;

    for i in start..candles.len() {
        let tr = (candles[i].high - candles[i].low)
            .max((candles[i].high - candles[i - 1].close).abs())
            .max((candles[i].low - candles[i - 1].close).abs());
        sum += tr;
    }

    sum / Decimal::from(period as i64)
}

/// Check for recent displacement
pub fn has_recent_displacement(
    candles: &[Candle],
    lookback: usize,
    direction: DisplacementDirection,
    min_atr_multiple: f64,
) -> Option<Displacement> {
    let displacements = detect_displacements(candles, "unknown", 2, 5, min_atr_multiple);

    displacements
        .into_iter()
        .filter(|d| candles.len().saturating_sub(d.end_index) <= lookback)
        .filter(|d| d.direction == direction)
        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
}

/// Get most recent displacement info
pub fn get_recent_displacement_info(candles: &[Candle]) -> (Option<DisplacementDirection>, f64) {
    let displacements = detect_displacements(candles, "unknown", 2, 5, 1.5);

    if let Some(recent) = displacements.last() {
        (Some(recent.direction), recent.strength)
    } else {
        (None, 0.0)
    }
}

/// Displacement bias analysis
#[derive(Debug, Clone)]
pub struct DisplacementBias {
    /// Overall bias from displacements
    pub bias: Option<DisplacementDirection>,
    /// Strength of bias (0.0 - 1.0)
    pub strength: f64,
    /// Recent displacement count
    pub recent_count: usize,
    /// Most significant displacement
    pub strongest: Option<Displacement>,
}

/// Analyze displacement patterns for bias
pub fn analyze_displacement_bias(candles: &[Candle]) -> DisplacementBias {
    let displacements = detect_displacements(candles, "unknown", 2, 5, 1.5);

    let lookback = 10;
    let recent: Vec<_> = displacements
        .iter()
        .filter(|d| candles.len().saturating_sub(d.end_index) <= lookback)
        .collect();

    let up_count = recent
        .iter()
        .filter(|d| d.direction == DisplacementDirection::Up)
        .count();
    let down_count = recent
        .iter()
        .filter(|d| d.direction == DisplacementDirection::Down)
        .count();

    let (bias, strength) = if up_count > down_count && up_count >= 2 {
        (
            Some(DisplacementDirection::Up),
            up_count as f64 / (up_count + down_count) as f64,
        )
    } else if down_count > up_count && down_count >= 2 {
        (
            Some(DisplacementDirection::Down),
            down_count as f64 / (up_count + down_count) as f64,
        )
    } else {
        (None, 0.0)
    };

    let strongest = displacements
        .iter()
        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap());

    DisplacementBias {
        bias,
        strength,
        recent_count: recent.len(),
        strongest: strongest.cloned(),
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
            Decimal::new(close + 20, 2),
            Decimal::new(open - 10, 2),
            Decimal::new(close, 2),
            1000,
        )
    }

    fn create_bearish_candle(open: i64, close: i64) -> Candle {
        Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(open, 2),
            Decimal::new(open + 10, 2),
            Decimal::new(close - 20, 2),
            Decimal::new(close, 2),
            1000,
        )
    }

    #[test]
    fn test_upward_displacement() {
        let candles = vec![
            create_bullish_candle(10000, 10200),
            create_bullish_candle(10200, 10400),
            create_bullish_candle(10400, 10600),
            create_bullish_candle(10600, 10800),
        ];

        let displacements = detect_displacements(&candles, "M15", 2, 5, 1.0);

        assert!(!displacements.is_empty());
        assert_eq!(displacements[0].direction, DisplacementDirection::Up);
    }

    #[test]
    fn test_displacement_strength() {
        let candles = vec![
            create_bullish_candle(10000, 10200),
            create_bullish_candle(10200, 10400),
            create_bullish_candle(10400, 10600),
            create_bullish_candle(10600, 10800),
        ];

        let displacements = detect_displacements(&candles, "M15", 2, 5, 1.0);

        if let Some(d) = displacements.first() {
            assert!(d.strength > 0.0);
            assert!(d.atr_multiple > 0.0);
        }
    }
}
