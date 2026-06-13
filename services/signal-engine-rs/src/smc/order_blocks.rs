//! Order Block (OB) detection
//!
//! Order blocks are institutional price levels where significant buying or selling
//! occurred, often leaving a footprint that acts as support/resistance.

use crate::market_data::Candle;
use crate::structure::swings::{SwingPoint, SwingType};
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Order Block structure
#[derive(Debug, Clone)]
pub struct OrderBlock {
    /// OB direction (bullish = buy zone, bearish = sell zone)
    pub direction: OBDirection,
    /// Index of OB candle
    pub index: usize,
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// Top of OB zone
    pub top: Decimal,
    /// Bottom of OB zone
    pub bottom: Decimal,
    /// OB open price (institutional entry)
    pub open: Decimal,
    /// OB close price
    pub close: Decimal,
    /// Strength score (0.0 - 1.0)
    pub strength: f64,
    /// Age in bars
    pub age_bars: u32,
    /// Has OB been mitigated (tested by price)
    pub mitigated: bool,
    /// Mitigation index (if any)
    pub mitigation_index: Option<usize>,
    /// Timeframe
    pub timeframe: String,
    /// Order block type
    pub ob_type: OBType,
}

/// OB direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OBDirection {
    /// Bullish order block (price should bounce up)
    Bullish,
    /// Bearish order block (price should reject down)
    Bearish,
}

/// Order block type classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OBType {
    /// Standard order block at swing point
    Standard,
    /// Breaker block (mitigated OB that reverses role)
    Breaker,
    /// Mitigation block (strong rejection)
    Mitigation,
}

/// Detect order blocks from price action
pub fn detect_order_blocks(
    candles: &[Candle],
    swings: &[SwingPoint],
    timeframe: &str,
) -> Vec<OrderBlock> {
    let mut obs = Vec::new();

    if candles.len() < 5 {
        return obs;
    }

    let swing_highs: Vec<&SwingPoint> = swings.iter()
        .filter(|s| s.swing_type == SwingType::High)
        .collect();
    let swing_lows: Vec<&SwingPoint> = swings.iter()
        .filter(|s| s.swing_type == SwingType::Low)
        .collect();

    // Find bullish order blocks (before bullish moves)
    for i in 3..candles.len() {
        let candle = &candles[i];

        // Look for bearish candle followed by strong bullish move
        let prior_candles = &candles[i.saturating_sub(3)..i];

        // Check if we have a bearish setup that led to bullish impulse
        if candle.is_bullish() && candle.close > candle.open {
            // Look for bearish order block candle
            if i >= 1 {
                let ob_candidate = &candles[i - 1];

                if ob_candidate.is_bearish() {
                    let strength = calculate_ob_strength(
                        ob_candidate,
                        candle,
                        &swing_lows,
                    );

                    if strength > 0.3 {
                        obs.push(OrderBlock {
                            direction: OBDirection::Bullish,
                            index: i - 1,
                            timestamp: ob_candidate.timestamp,
                            top: ob_candidate.open.max(ob_candidate.close),
                            bottom: ob_candidate.open.min(ob_candidate.close),
                            open: ob_candidate.open,
                            close: ob_candidate.close,
                            strength,
                            age_bars: (candles.len() - (i - 1)) as u32,
                            mitigated: false,
                            mitigation_index: None,
                            timeframe: timeframe.to_string(),
                            ob_type: classify_ob_type(ob_candidate, &candles[i..i.min(candles.len())]),
                        });
                    }
                }
            }
        }

        // Find bearish order blocks
        if candle.is_bearish() {
            if i >= 1 {
                let ob_candidate = &candles[i - 1];

                if ob_candidate.is_bullish() {
                    let strength = calculate_ob_strength(
                        ob_candidate,
                        candle,
                        &swing_highs,
                    );

                    if strength > 0.3 {
                        obs.push(OrderBlock {
                            direction: OBDirection::Bearish,
                            index: i - 1,
                            timestamp: ob_candidate.timestamp,
                            top: ob_candidate.open.max(ob_candidate.close),
                            bottom: ob_candidate.open.min(ob_candidate.close),
                            open: ob_candidate.open,
                            close: ob_candidate.close,
                            strength,
                            age_bars: (candles.len() - (i - 1)) as u32,
                            mitigated: false,
                            mitigation_index: None,
                            timeframe: timeframe.to_string(),
                            ob_type: classify_ob_type(ob_candidate, &candles[i..i.min(candles.len())]),
                        });
                    }
                }
            }
        }
    }

    // Mark mitigated OBs
    mark_mitigated(&mut obs, candles);

    obs
}

fn calculate_ob_strength(
    ob_candle: &Candle,
    impulse_candle: &Candle,
    relevant_swings: &[&SwingPoint],
) -> f64 {
    let mut score = 0.0;

    // Factor 1: Size of impulsive move
    let impulse_size = (impulse_candle.close - impulse_candle.open).abs();
    let ob_range = ob_candle.high - ob_candle.low;

    if ob_range > Decimal::ZERO {
        let ratio = impulse_size / ob_range;
        score += ratio.min(Decimal::from(3)).to_f64().unwrap_or(0.0) * 0.25;
    }

    // Factor 2: Imbalance (large wick in OB candle)
    let body = (ob_candle.close - ob_candle.open).abs();
    let total_range = ob_candle.high - ob_candle.low;

    if total_range > Decimal::ZERO {
        let wick_ratio = (total_range - body) / total_range;
        if wick_ratio > Decimal::from_f64_retain(0.5).unwrap_or_default() {
            score += 0.25;
        }
    }

    // Factor 3: Proximity to swing
    let ob_mid = (ob_candle.high + ob_candle.low) / Decimal::from(2);
    for swing in relevant_swings {
        let distance = (swing.price - ob_mid).abs() / ob_range.max(Decimal::new(1, 4));
        if distance < Decimal::from(2) {
            score += 0.25;
            break;
        }
    }

    // Factor 4: Freshness (recent OBs score higher)
    score += 0.25;

    score.min(1.0)
}

fn classify_ob_type(ob_candle: &Candle, following_candles: &[Candle]) -> OBType {
    // Simplified classification
    if following_candles.len() >= 2 {
        let immediate_move = (following_candles[0].close - following_candles[0].open).abs();
        let ob_range = ob_candle.high - ob_candle.low;

        if immediate_move > ob_range * Decimal::from(2) {
            return OBType::Mitigation;
        }
    }

    OBType::Standard
}

fn mark_mitigated(obs: &mut [OrderBlock], candles: &[Candle]) {
    for ob in obs.iter_mut() {
        for i in (ob.index + 1)..candles.len() {
            let candle = &candles[i];

            match ob.direction {
                OBDirection::Bullish => {
                    // Mitigated if price trades into the OB zone
                    if candle.low <= ob.top && candle.low >= ob.bottom {
                        ob.mitigated = true;
                        ob.mitigation_index = Some(i);
                        break;
                    }
                    // Or if price clearly violates it
                    if candle.close < ob.bottom {
                        ob.mitigated = true;
                        ob.mitigation_index = Some(i);
                        break;
                    }
                }
                OBDirection::Bearish => {
                    if candle.high >= ob.bottom && candle.high <= ob.top {
                        ob.mitigated = true;
                        ob.mitigation_index = Some(i);
                        break;
                    }
                    if candle.close > ob.top {
                        ob.mitigated = true;
                        ob.mitigation_index = Some(i);
                        break;
                    }
                }
            }
        }
    }
}

/// Find unmitigated order blocks
pub fn find_fresh_obs(obs: &[OrderBlock], max_age: u32) -> Vec<&OrderBlock> {
    obs.iter()
        .filter(|ob| !ob.mitigated)
        .filter(|ob| ob.age_bars <= max_age)
        .collect()
}

/// Find OBs nearest to current price
pub fn find_nearest_obs(
    obs: &[OrderBlock],
    current_price: Decimal,
) -> (Option<&OrderBlock>, Option<&OrderBlock>) {
    let fresh = find_fresh_obs(obs, 50);

    let bullish = fresh.iter()
        .filter(|ob| ob.direction == OBDirection::Bullish)
        .min_by_key(|ob| (ob.bottom - current_price).abs());

    let bearish = fresh.iter()
        .filter(|ob| ob.direction == OBDirection::Bearish)
        .min_by_key(|ob| (ob.top - current_price).abs());

    (bullish.copied(), bearish.copied())
}

/// Get OB zone for entry consideration
pub fn get_entry_zone(ob: &OrderBlock) -> (Decimal, Decimal) {
    // Extend zone slightly for entries
    let extension = (ob.top - ob.bottom) * Decimal::from_f64_retain(0.1).unwrap_or_default();

    match ob.direction {
        OBDirection::Bullish => (ob.bottom - extension, ob.top),
        OBDirection::Bearish => (ob.bottom, ob.top + extension),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candle_direction(bullish: bool, open: i64, close: i64) -> Candle {
        let high = if bullish { close + 10 } else { open + 10 };
        let low = if bullish { open - 10 } else { close - 10 };

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
    fn test_order_block_detection() {
        let candles = vec![
            create_candle_direction(false, 10500, 10400), // Bearish OB candidate
            create_candle_direction(true, 10400, 10600),  // Strong bullish impulse
        ];

        let swings = vec![];

        let obs = detect_order_blocks(&candles, &swings, "M15");

        assert!(!obs.is_empty());
        assert_eq!(obs[0].direction, OBDirection::Bullish);
    }
}
