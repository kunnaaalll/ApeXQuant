//! Volatility measurement and risk adjustment
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Volatility regime classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VolatilityRegime {
    /// Very low volatility (< 20% of normal)
    VeryLow,
    /// Low volatility (20-60% of normal)
    Low,
    /// Normal volatility (60-140% of normal)
    Normal,
    /// High volatility (140-200% of normal)
    High,
    /// Very high volatility (200-300% of normal)
    VeryHigh,
    /// Extreme volatility (> 300% of normal)
    Extreme,
}

impl VolatilityRegime {
    /// Get risk adjustment factor for this regime
    pub fn risk_factor(&self) -> Decimal {
        match self {
            VolatilityRegime::VeryLow => Decimal::from_f64(1.3).unwrap(),
            VolatilityRegime::Low => Decimal::from_f64(1.15).unwrap(),
            VolatilityRegime::Normal => Decimal::ONE,
            VolatilityRegime::High => Decimal::from_f64(0.75).unwrap(),
            VolatilityRegime::VeryHigh => Decimal::from_f64(0.5).unwrap(),
            VolatilityRegime::Extreme => Decimal::from_f64(0.25).unwrap(),
        }
    }

    /// Whether trading should be avoided
    pub fn avoid_trading(&self) -> bool {
        matches!(self, VolatilityRegime::Extreme)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            VolatilityRegime::VeryLow => "Very low volatility",
            VolatilityRegime::Low => "Low volatility",
            VolatilityRegime::Normal => "Normal volatility",
            VolatilityRegime::High => "High volatility",
            VolatilityRegime::VeryHigh => "Very high volatility",
            VolatilityRegime::Extreme => "Extreme volatility - caution advised",
        }
    }
}

/// Volatility metrics for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    /// Current volatility regime
    pub regime: VolatilityRegime,
    /// ATR value
    pub atr: Decimal,
    /// Volatility relative to historical average (> 1 = higher than normal)
    pub relative_volatility: Decimal,
    /// Spread expansion factor (current spread / normal spread)
    pub spread_expansion: Decimal,
    /// Timestamp
    pub timestamp: OffsetDateTime,
}

/// Volatility measurement engine
pub struct VolatilityEngine {
    /// ATR period for calculation
    atr_period: usize,
    /// Historical ATR for comparison
    historical_atr: Decimal,
    /// Spread expansion warning threshold
    spread_warning_threshold: Decimal,
}

impl VolatilityEngine {
    /// Create new volatility engine
    pub fn new() -> Self {
        Self {
            atr_period: 14,
            historical_atr: Decimal::ZERO,
            spread_warning_threshold: Decimal::from(2),
        }
    }

    /// Analyze current volatility conditions
    pub fn analyze(&self, atr: Option<Decimal>, spread: Decimal) -> VolatilityMetrics {
        let atr_value = atr.unwrap_or(Decimal::ZERO);
        let now = OffsetDateTime::now_utc();

        // Determine regime based on ATR vs historical
        let regime = self.classify_regime(atr_value);

        // Calculate relative volatility
        let relative_volatility = if self.historical_atr > Decimal::ZERO {
            atr_value / self.historical_atr
        } else {
            Decimal::ONE
        };

        // Calculate spread expansion
        // Simplified: assume normal spread is 1 pip (0.0001 for forex pairs)
        let normal_spread = Decimal::from_f64(0.0001).unwrap_or(Decimal::ONE);
        let spread_expansion = if normal_spread > Decimal::ZERO {
            spread / normal_spread
        } else {
            Decimal::ONE
        };

        VolatilityMetrics {
            regime,
            atr: atr_value,
            relative_volatility,
            spread_expansion,
            timestamp: now,
        }
    }

    /// Get position size adjustment for current volatility
    pub fn position_adjustment(&self, metrics: &VolatilityMetrics) -> Decimal {
        let base = metrics.regime.risk_factor();

        // Additional spread-based reduction
        let spread_factor = if metrics.spread_expansion > self.spread_warning_threshold {
            Decimal::from_f64(0.8).unwrap()
        } else {
            Decimal::ONE
        };

        base * spread_factor
    }

    /// Check if stop distance should be widened
    pub fn recommend_stop_adjustment(&self, metrics: &VolatilityMetrics) -> Decimal {
        match metrics.regime {
            VolatilityRegime::VeryLow => Decimal::from_f64(0.8).unwrap(),
            VolatilityRegime::Low => Decimal::from_f64(0.9).unwrap(),
            VolatilityRegime::Normal => Decimal::ONE,
            VolatilityRegime::High => Decimal::from_f64(1.2).unwrap(),
            VolatilityRegime::VeryHigh => Decimal::from_f64(1.5).unwrap(),
            VolatilityRegime::Extreme => Decimal::from_f64(2.0).unwrap(),
        }
    }

    /// Set historical ATR for relative calculations
    pub fn set_historical_atr(&mut self, atr: Decimal) {
        self.historical_atr = atr;
    }

    fn classify_regime(&self, atr: Decimal) -> VolatilityRegime {
        if self.historical_atr <= Decimal::ZERO {
            return VolatilityRegime::Normal;
        }

        let ratio = atr / self.historical_atr;

        if ratio < Decimal::from_f64(0.2).unwrap() {
            VolatilityRegime::VeryLow
        } else if ratio < Decimal::from_f64(0.6).unwrap() {
            VolatilityRegime::Low
        } else if ratio < Decimal::from_f64(1.4).unwrap() {
            VolatilityRegime::Normal
        } else if ratio < Decimal::from_f64(2.0).unwrap() {
            VolatilityRegime::High
        } else if ratio < Decimal::from_f64(3.0).unwrap() {
            VolatilityRegime::VeryHigh
        } else {
            VolatilityRegime::Extreme
        }
    }

    /// Calculate ATR from price series
    pub fn calculate_atr(&self, highs: &[Decimal], lows: &[Decimal], closes: &[Decimal]) -> Decimal {
        if highs.len() < 2 || lows.len() < 2 || closes.len() < 2 {
            return Decimal::ZERO;
        }

        let mut tr_sum = Decimal::ZERO;
        let count = highs.len().min(lows.len()).min(closes.len());

        for i in 1..count {
            let high_low = highs[i] - lows[i];
            let high_close = (highs[i] - closes[i - 1]).abs();
            let low_close = (lows[i] - closes[i - 1]).abs();

            let tr = high_low.max(high_close).max(low_close);
            tr_sum += tr;
        }

        if count > 1 {
            tr_sum / Decimal::from((count - 1) as u32)
        } else {
            Decimal::ZERO
        }
    }

    /// Detect volatility spike (sudden increase)
    pub fn is_volatility_spike(&self, current_atr: Decimal, threshold_multiple: Decimal) -> bool {
        self.historical_atr > Decimal::ZERO
            && current_atr > self.historical_atr * threshold_multiple
    }
}

impl Default for VolatilityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_regime_factors() {
        assert_eq!(VolatilityRegime::Normal.risk_factor(), Decimal::ONE);
        assert!(VolatilityRegime::High.risk_factor() < Decimal::ONE);
        assert!(VolatilityRegime::Low.risk_factor() > Decimal::ONE);
    }

    #[test]
    fn test_extreme_avoidance() {
        assert!(VolatilityRegime::Extreme.avoid_trading());
        assert!(!VolatilityRegime::High.avoid_trading());
    }

    #[test]
    fn test_regime_classification() {
        let mut engine = VolatilityEngine::new();
        engine.set_historical_atr(Decimal::from_f64(0.001).unwrap());

        // Very low ATR
        let metrics = engine.analyze(Some(Decimal::from_f64(0.0001).unwrap()), Decimal::ZERO);
        assert_eq!(metrics.regime, VolatilityRegime::VeryLow);

        // Normal ATR
        let metrics = engine.analyze(Some(Decimal::from_f64(0.001).unwrap()), Decimal::ZERO);
        assert_eq!(metrics.regime, VolatilityRegime::Normal);

        // Very high ATR
        let metrics = engine.analyze(Some(Decimal::from_f64(0.004).unwrap()), Decimal::ZERO);
        assert_eq!(metrics.regime, VolatilityRegime::VeryHigh);
    }

    #[test]
    fn test_atr_calculation() {
        let engine = VolatilityEngine::new();

        let highs = vec![
            Decimal::from_f64(1.1000).unwrap(),
            Decimal::from_f64(1.1010).unwrap(),
            Decimal::from_f64(1.1020).unwrap(),
        ];
        let lows = vec![
            Decimal::from_f64(1.0980).unwrap(),
            Decimal::from_f64(1.0990).unwrap(),
            Decimal::from_f64(1.1000).unwrap(),
        ];
        let closes = vec![
            Decimal::from_f64(1.0990).unwrap(),
            Decimal::from_f64(1.1000).unwrap(),
            Decimal::from_f64(1.1010).unwrap(),
        ];

        let atr = engine.calculate_atr(&highs, &lows, &closes);
        assert!(atr > Decimal::ZERO);
    }
}
