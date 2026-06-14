//! Session overlap analysis
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::MathematicalOps;

use rust_decimal::Decimal;
use time::{OffsetDateTime, Time};

/// Session overlap characteristics
pub struct SessionOverlap;

impl SessionOverlap {
    /// Create new overlap analyzer
    pub fn new() -> Self {
        Self
    }

    /// London-NY overlap (highest liquidity)
    pub fn london_ny_overlap() -> (Time, Time) {
        (
            Time::from_hms(14, 0, 0).unwrap(), // NY open
            Time::from_hms(17, 0, 0).unwrap(), // London close
        )
    }

    /// Sydney-Asia overlap
    pub fn asia_overlap() -> (Time, Time) {
        (
            Time::from_hms(23, 0, 0).unwrap(), // Tokyo open
            Time::from_hms(7, 0, 0).unwrap(),  // Tokyo close
        )
    }

    /// Check if in London-NY overlap
    pub fn is_london_ny(&self, timestamp: OffsetDateTime) -> bool {
        let (start, end) = Self::london_ny_overlap();
        let time = timestamp.time();
        time >= start && time < end
    }

    /// Check if in early London (overlap with Asia close)
    pub fn is_london_early(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        // 07:00 - 09:00 London early
        hour >= 7 && hour < 9
    }

    /// Get overlap type
    pub fn overlap_type(&self, timestamp: OffsetDateTime) -> OverlapType {
        if self.is_london_ny(timestamp) {
            OverlapType::LondonNewYork
        } else if self.is_early_overlap(timestamp) {
            OverlapType::EarlyOverlap
        } else if self.is_late_overlap(timestamp) {
            OverlapType::LateOverlap
        } else {
            OverlapType::None
        }
    }

    /// Characteristics by overlap type
    pub fn characteristics(&self, overlap: OverlapType) -> OverlapCharacteristics {
        match overlap {
            OverlapType::LondonNewYork => OverlapCharacteristics {
                liquidity: Decimal::from_f64(1.0).unwrap(),
                volatility: Decimal::from_f64(1.2).unwrap(),
                spread_tightness: Decimal::from_f64(0.9).unwrap(),
                trend_probability: Decimal::from_f64(0.75).unwrap(),
                recommended: true,
            },
            OverlapType::EarlyOverlap => OverlapCharacteristics {
                liquidity: Decimal::from_f64(0.7).unwrap(),
                volatility: Decimal::from_f64(1.0).unwrap(),
                spread_tightness: Decimal::from_f64(1.1).unwrap(),
                trend_probability: Decimal::from_f64(0.6).unwrap(),
                recommended: true,
            },
            OverlapType::LateOverlap => OverlapCharacteristics {
                liquidity: Decimal::from_f64(0.5).unwrap(),
                volatility: Decimal::from_f64(1.1).unwrap(),
                spread_tightness: Decimal::from_f64(1.3).unwrap(),
                trend_probability: Decimal::from_f64(0.55).unwrap(),
                recommended: false,
            },
            OverlapType::None => OverlapCharacteristics {
                liquidity: Decimal::from_f64(0.6).unwrap(),
                volatility: Decimal::from_f64(0.9).unwrap(),
                spread_tightness: Decimal::from_f64(1.2).unwrap(),
                trend_probability: Decimal::from_f64(0.5).unwrap(),
                recommended: true,
            },
        }
    }

    fn is_early_overlap(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        // 07:00 - 09:00 (Frankfurt-London)
        hour >= 7 && hour < 9
    }

    fn is_late_overlap(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        // 20:00 - 22:00 (Late NY)
        hour >= 20 && hour < 22
    }

    /// Risk adjustment for overlap period
    pub fn risk_adjustment(&self, overlap: OverlapType) -> Decimal {
        let chars = self.characteristics(overlap);

        if !chars.recommended {
            return Decimal::from_f64(0.7).unwrap();
        }

        // Higher liquidity = can take slightly more risk
        // Higher volatility = reduce size
        let liquidity_factor = chars.liquidity;
        let vol_adjustment = Decimal::ONE / chars.volatility.sqrt().unwrap_or(Decimal::ONE);

        (liquidity_factor + vol_adjustment) / Decimal::from(2)
    }
}

impl Default for SessionOverlap {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of session overlap
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlapType {
    /// London-NY overlap (peak liquidity)
    LondonNewYork,
    /// Early session overlap
    EarlyOverlap,
    /// Late session overlap (declining liquidity)
    LateOverlap,
    /// No overlap
    None,
}

/// Characteristics of an overlap period
#[derive(Debug, Clone)]
pub struct OverlapCharacteristics {
    /// Liquidity level (0-1)
    pub liquidity: Decimal,
    /// Volatility multiplier
    pub volatility: Decimal,
    /// Spread relative to normal
    pub spread_tightness: Decimal,
    /// Probability of sustained trend
    pub trend_probability: Decimal,
    /// Whether trading is recommended
    pub recommended: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_london_ny_overlap_times() {
        let (start, end) = SessionOverlap::london_ny_overlap();
        assert_eq!(start.hour(), 14);
        assert_eq!(end.hour(), 17);
    }

    #[test]
    fn test_overlap_characteristics() {
        let overlap = SessionOverlap::new();
        let chars = overlap.characteristics(OverlapType::LondonNewYork);

        assert!(chars.liquidity > chars.volatility);
        assert!(chars.recommended);
    }

    #[test]
    fn test_risk_adjustment() {
        let overlap = SessionOverlap::new();

        let adjustment = overlap.risk_adjustment(OverlapType::LondonNewYork);
        assert!(adjustment > Decimal::from_f64(0.5).unwrap());

        let late = overlap.risk_adjustment(OverlapType::LateOverlap);
        assert!(!overlap.characteristics(OverlapType::LateOverlap).recommended);
    }
}
