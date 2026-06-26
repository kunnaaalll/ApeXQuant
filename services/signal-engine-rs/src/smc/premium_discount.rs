//! Premium and Discount zone analysis
//!
//! Premium zones are above the equilibrium (midpoint) where selling is favorable.
//! Discount zones are below the equilibrium where buying is favorable.
use num_traits::ToPrimitive;

use crate::market_data::Candle;
use rust_decimal::Decimal;

/// Premium/Discount analysis result
#[derive(Debug, Clone)]
pub struct PremiumDiscount {
    /// Current zone classification
    pub zone: PriceZone,
    /// Distance from equilibrium (0.0 = at equilibrium, 1.0 = at extreme)
    pub distance_from_eq: f64,
    /// Premium/discount score (-1.0 to 1.0)
    pub score: f64,
    /// Equilibrium level (midpoint)
    pub equilibrium: Decimal,
    /// Upper boundary (premium starts)
    pub premium_start: Decimal,
    /// Lower boundary (discount starts)
    pub discount_start: Decimal,
    /// Recent range
    pub range: Decimal,
}

/// Price zone classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriceZone {
    /// Deep premium (top 10% of range)
    DeepPremium,
    /// Premium zone (50-90% of range)
    Premium,
    /// Near equilibrium (40-60% of range)
    Equilibrium,
    /// Discount zone (10-50% of range)
    Discount,
    /// Deep discount (bottom 10% of range)
    DeepDiscount,
}

impl PriceZone {
    /// Check if zone is favorable for longs
    pub fn favors_longs(&self) -> bool {
        matches!(self, PriceZone::Discount | PriceZone::DeepDiscount)
    }

    /// Check if zone is favorable for shorts
    pub fn favors_shorts(&self) -> bool {
        matches!(self, PriceZone::Premium | PriceZone::DeepPremium)
    }

    /// Get zone bias score (-1.0 to 1.0)
    pub fn bias_score(&self) -> f64 {
        match self {
            PriceZone::DeepPremium => -1.0,
            PriceZone::Premium => -0.5,
            PriceZone::Equilibrium => 0.0,
            PriceZone::Discount => 0.5,
            PriceZone::DeepDiscount => 1.0,
        }
    }
}

/// Calculate premium/discount from recent price action
pub fn calculate_premium_discount(candles: &[Candle]) -> PremiumDiscount {
    if candles.len() < 10 {
        let last_close = candles.last().map(|c| c.close).unwrap_or_default();
        return PremiumDiscount {
            zone: PriceZone::Equilibrium,
            distance_from_eq: 0.0,
            score: 0.0,
            equilibrium: last_close,
            premium_start: last_close,
            discount_start: last_close,
            range: Decimal::ZERO,
        };
    }

    let highs: Vec<Decimal> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<Decimal> = candles.iter().map(|c| c.low).collect();

    let highest = highs.iter().fold(Decimal::MIN, |a, &b| a.max(b));
    let lowest = lows.iter().fold(Decimal::MAX, |a, &b| a.min(b));
    let range = highest - lowest;

    let equilibrium = (highest + lowest) / Decimal::from(2);

    let current_price = candles[candles.len() - 1].close;

    // Define zone boundaries
    let premium_start = equilibrium + range * Decimal::from_f64_retain(0.1).unwrap_or_default();
    let deep_premium = highest - range * Decimal::from_f64_retain(0.1).unwrap_or_default();
    let discount_start = equilibrium - range * Decimal::from_f64_retain(0.1).unwrap_or_default();
    let deep_discount = lowest + range * Decimal::from_f64_retain(0.1).unwrap_or_default();

    // Determine zone
    let zone = if current_price >= deep_premium {
        PriceZone::DeepPremium
    } else if current_price >= premium_start {
        PriceZone::Premium
    } else if current_price > discount_start {
        PriceZone::Equilibrium
    } else if current_price > deep_discount {
        PriceZone::Discount
    } else {
        PriceZone::DeepDiscount
    };

    // Calculate distance from equilibrium
    let distance_pct = if range > Decimal::ZERO {
        ((current_price - equilibrium).abs() / (range / Decimal::from(2)))
            .min(Decimal::ONE)
            .to_f64()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let score = zone.bias_score() * distance_pct;

    PremiumDiscount {
        zone,
        distance_from_eq: distance_pct,
        score,
        equilibrium,
        premium_start,
        discount_start,
        range,
    }
}

/// Check if price is seeking equilibrium
pub fn is_seeking_equilibrium(
    candles: &[Candle],
    direction: SeekingDirection,
) -> bool {
    let pd = calculate_premium_discount(candles);

    match direction {
        SeekingDirection::ToEquilibrium => {
            // Price moving toward equilibrium
            let recent = &candles[candles.len().saturating_sub(5)..];
            if recent.len() < 2 {
                return false;
            }

            let start_price = recent[0].close;
            let end_price = recent[recent.len() - 1].close;

            // In premium and moving down, or in discount and moving up
            match pd.zone {
                PriceZone::Premium | PriceZone::DeepPremium => {
                    end_price < start_price && end_price < pd.equilibrium + pd.range * Decimal::from_f64_retain(0.05).unwrap_or_default()
                }
                PriceZone::Discount | PriceZone::DeepDiscount => {
                    end_price > start_price && end_price > pd.equilibrium - pd.range * Decimal::from_f64_retain(0.05).unwrap_or_default()
                }
                _ => false,
            }
        }
        SeekingDirection::AwayFromEquilibrium => {
            // Price moving away suggests trend continuation
            let recent = &candles[candles.len().saturating_sub(5)..];
            if recent.len() < 2 {
                return false;
            }

            let start_price = recent[0].close;
            let end_price = recent[recent.len() - 1].close;

            match pd.zone {
                PriceZone::Premium | PriceZone::DeepPremium => {
                    end_price > start_price
                }
                PriceZone::Discount | PriceZone::DeepDiscount => {
                    end_price < start_price
                }
                _ => false,
            }
        }
    }
}

/// Direction of equilibrium seeking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeekingDirection {
    /// Price moving toward equilibrium
    ToEquilibrium,
    /// Price moving away from equilibrium
    AwayFromEquilibrium,
}

/// Optimize entry within zone
pub fn optimize_entry_in_zone(
    candles: &[Candle],
    zone: PriceZone,
    entry_price: Decimal,
) -> (Decimal, f64) {
    let pd = calculate_premium_discount(candles);

    let optimized_entry = match zone {
        PriceZone::DeepDiscount => {
            // Wait for deeper discount or take current
            let ideal = pd.discount_start - pd.range * Decimal::from_f64_retain(0.05).unwrap_or_default();
            entry_price.min(ideal)
        }
        PriceZone::Discount => {
            // Current entry is reasonable
            entry_price
        }
        PriceZone::Equilibrium => {
            // May want to wait for better zone
            entry_price
        }
        PriceZone::Premium => {
            entry_price
        }
        PriceZone::DeepPremium => {
            let ideal = pd.premium_start + pd.range * Decimal::from_f64_retain(0.05).unwrap_or_default();
            entry_price.max(ideal)
        }
    };

    let improvement = if entry_price != Decimal::ZERO {
        ((optimized_entry - entry_price).abs() / entry_price)
            .to_f64()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    (optimized_entry, improvement)
}

/// Premium/Discount analysis summary
#[derive(Debug, Clone)]
pub struct PremiumDiscountAnalysis {
    /// Current zone
    pub zone: PriceZone,
    /// Zone score
    pub score: f64,
    /// Favors longs
    pub favors_longs: bool,
    /// Favors shorts
    pub favors_shorts: bool,
    /// Seeking equilibrium
    pub seeking_equilibrium: bool,
}

/// Analyze premium/discount conditions
pub fn analyze_premium_discount(candles: &[Candle]) -> PremiumDiscountAnalysis {
    let pd = calculate_premium_discount(candles);
    let seeking = is_seeking_equilibrium(candles, SeekingDirection::ToEquilibrium);

    PremiumDiscountAnalysis {
        zone: pd.zone,
        score: pd.score,
        favors_longs: pd.zone.favors_longs(),
        favors_shorts: pd.zone.favors_shorts(),
        seeking_equilibrium: seeking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_candles_with_range(low: i64, high: i64, current: i64) -> Vec<Candle> {
        let mut candles = Vec::new();

        // Create candles spanning the range
        for i in 0..10 {
            let price = low + (high - low) * i / 10;
            candles.push(Candle::new(
                OffsetDateTime::now_utc(),
                Decimal::new(price, 2),
                Decimal::new(price + 50, 2),
                Decimal::new(price - 50, 2),
                Decimal::new(price, 2),
                1000,
            ));
        }

        // Final candle at current price
        candles.push(Candle::new(
            OffsetDateTime::now_utc(),
            Decimal::new(current, 2),
            Decimal::new(current + 50, 2),
            Decimal::new(current - 50, 2),
            Decimal::new(current, 2),
            1000,
        ));

        candles
    }

    #[test]
    fn test_premium_zone() {
        let candles = create_candles_with_range(10000, 11000, 10800);
        let pd = calculate_premium_discount(&candles);

        assert!(matches!(pd.zone, PriceZone::Premium | PriceZone::DeepPremium));
        assert!(pd.score < 0.0);
    }

    #[test]
    fn test_discount_zone() {
        let candles = create_candles_with_range(10000, 11000, 10200);
        let pd = calculate_premium_discount(&candles);

        assert!(matches!(pd.zone, PriceZone::Discount | PriceZone::DeepDiscount));
        assert!(pd.score > 0.0);
    }
}
