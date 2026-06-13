//! Order flow imbalance detection
//!
//! Detects areas of significant buying or selling pressure that create
//! directional bias in price action.

use crate::market_data::Candle;
use rust_decimal::Decimal;

/// Order flow imbalance
#[derive(Debug, Clone)]
pub struct Imbalance {
    /// Direction of imbalance
    pub direction: ImbalanceDirection,
    /// Start index
    pub start_index: usize,
    /// End index
    pub end_index: usize,
    /// Cumulative delta
    pub delta: Decimal,
    /// Average candle size
    pub avg_candle_size: Decimal,
    /// Strength (0.0 - 1.0)
    pub strength: f64,
    /// Number of bars in imbalance
    pub bars: u32,
}

/// Imbalance direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImbalanceDirection {
    /// Buying pressure
    Buying,
    /// Selling pressure
    Selling,
    /// Neutral
    Neutral,
}

/// Detect order flow imbalance from candles
pub fn detect_imbalance(
    candles: &[Candle],
    min_bars: usize,
    lookback: usize,
) -> Vec<Imbalance> {
    let mut imbalances = Vec::new();

    if candles.len() < min_bars {
        return imbalances;
    }

    let start = candles.len().saturating_sub(lookback);

    for i in start..candles.len() - min_bars + 1 {
        let window = &candles[i..(i + min_bars).min(candles.len())];

        if let Some(imbalance) = analyze_window(window, i) {
            imbalances.push(imbalance);
        }
    }

    imbalances
}

fn analyze_window(candles: &[Candle], start_idx: usize) -> Option<Imbalance> {
    if candles.len() < 3 {
        return None;
    }

    let bullish_count = candles.iter().filter(|c| c.is_bullish()).count();
    let bearish_count = candles.iter().filter(|c| c.is_bearish()).count();
    let total = candles.len();

    let direction = if bullish_count >= total * 2 / 3 {
        ImbalanceDirection::Buying
    } else if bearish_count >= total * 2 / 3 {
        ImbalanceDirection::Selling
    } else {
        ImbalanceDirection::Neutral
    };

    if matches!(direction, ImbalanceDirection::Neutral) {
        return None;
    }

    // Calculate delta (approximate using close - open accumulation)
    let delta = candles.iter()
        .map(|c| c.close - c.open)
        .sum::<Decimal>();

    let avg_size = candles.iter()
        .map(|c| c.range())
        .fold(Decimal::ZERO, |acc, r| acc + r)
        / Decimal::from(candles.len() as i64);

    let strength = calculate_imbalance_strength(candles, &direction, delta, avg_size);

    Some(Imbalance {
        direction,
        start_index: start_idx,
        end_index: start_idx + candles.len() - 1,
        delta: delta.abs(),
        avg_candle_size: avg_size,
        strength,
        bars: candles.len() as u32,
    })
}

fn calculate_imbalance_strength(
    candles: &[Candle],
    direction: &ImbalanceDirection,
    delta: Decimal,
    avg_size: Decimal,
) -> f64 {
    let mut score = 0.0;

    let directional_matches = candles.iter().filter(|c| {
        match direction {
            ImbalanceDirection::Buying => c.is_bullish(),
            ImbalanceDirection::Selling => c.is_bearish(),
            _ => false,
        }
    }).count() as f64;

    let consistency = directional_matches / candles.len() as f64;
    score += consistency * 0.4;

    if avg_size > Decimal::ZERO {
        let delta_candles = (delta.abs() / avg_size).to_f64().unwrap_or(0.0);
        score += (delta_candles / candles.len() as f64).min(0.4);
    }

    let body_sum: Decimal = candles.iter().map(|c| c.body_size()).sum();
    let total_range: Decimal = candles.iter().map(|c| c.range()).sum();

    if total_range > Decimal::ZERO {
        let body_ratio = body_sum / total_range;
        score += body_ratio.to_f64().unwrap_or(0.0) * 0.2;
    }

    score
}

/// Current imbalance based on recent candles
pub fn current_imbalance(candles: &[Candle], period: usize) -> (ImbalanceDirection, f64) {
    if candles.len() < period {
        return (ImbalanceDirection::Neutral, 0.0);
    }

    let recent = &candles[candles.len() - period..];
    let bullish = recent.iter().filter(|c| c.is_bullish()).count();
    let bearish = recent.iter().filter(|c| c.is_bearish()).count();

    let total = recent.len() as f64;
    let bullish_ratio = bullish as f64 / total;
    let bearish_ratio = bearish as f64 / total;

    if bullish_ratio > 0.7 {
        (ImbalanceDirection::Buying, bullish_ratio)
    } else if bearish_ratio > 0.7 {
        (ImbalanceDirection::Selling, bearish_ratio)
    } else {
        (ImbalanceDirection::Neutral, 0.5)
    }
}

/// Imbalance analysis
#[derive(Debug, Clone)]
pub struct ImbalanceAnalysis {
    /// Current direction
    pub current: ImbalanceDirection,
    /// Current strength
    pub strength: f64,
    /// Recent imbalances
    pub recent_imbalances: Vec<Imbalance>,
    /// Dominant bias over period
    pub dominant_bias: ImbalanceDirection,
}

/// Analyze imbalance conditions
pub fn analyze_imbalance(candles: &[Candle]) -> ImbalanceAnalysis {
    let (current, strength) = current_imbalance(candles, 10);
    let recent = detect_imbalance(candles, 3, 30);

    let buying_count = recent.iter()
        .filter(|i| i.direction == ImbalanceDirection::Buying)
        .count();
    let selling_count = recent.iter()
        .filter(|i| i.direction == ImbalanceDirection::Selling)
        .count();

    let dominant = if buying_count > selling_count {
        ImbalanceDirection::Buying
    } else if selling_count > buying_count {
        ImbalanceDirection::Selling
    } else {
        ImbalanceDirection::Neutral
    };

    ImbalanceAnalysis {
        current,
        strength,
        recent_imbalances: recent,
        dominant_bias: dominant,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candle_direction(bullish: bool, open: i64, close: i64) -> Candle {
        let (high, low) = if bullish {
            (close + 10, open - 10)
        } else {
            (open + 10, close - 10)
        };

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
    fn test_buying_imbalance() {
        let candles = vec![
            create_candle_direction(true, 10000, 10100),
            create_candle_direction(true, 10100, 10200),
            create_candle_direction(true, 10200, 10300),
            create_candle_direction(true, 10300, 10400),
        ];

        let imbalances = detect_imbalance(&candles, 3, 10);

        assert!(!imbalances.is_empty());
        assert_eq!(imbalances[0].direction, ImbalanceDirection::Buying);
    }

    #[test]
    fn test_current_imbalance() {
        let candles = vec![
            create_candle_direction(true, 10000, 10100),
            create_candle_direction(true, 10100, 10200),
            create_candle_direction(true, 10200, 10300),
        ];

        let (direction, strength) = current_imbalance(&candles, 3);

        assert_eq!(direction, ImbalanceDirection::Buying);
        assert!(strength > 0.9);
    }
}
