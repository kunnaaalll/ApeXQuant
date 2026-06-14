//! Confidence scoring and mapping to risk profiles
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Inputs to confidence calculation
#[derive(Debug, Clone, Copy)]
pub struct ConfidenceInputs {
    /// Signal confidence (0-1)
    pub signal_confidence: Decimal,
    /// Confluence score (0-10)
    pub confluence_score: Decimal,
    /// Regime quality (0-1)
    pub regime_quality: Decimal,
    /// Pattern quality (0-1)
    pub pattern_quality: Decimal,
    /// Session quality (0-1)
    pub session_quality: Decimal,
}

/// Result of confidence calculation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceScore {
    /// Overall confidence (0-1)
    pub overall: Decimal,
    /// Signal component
    pub signal: Decimal,
    /// Confluence component
    pub confluence: Decimal,
    /// Regime component
    pub regime: Decimal,
    /// Market condition quality
    pub market_condition_quality: Decimal,
}

impl ConfidenceScore {
    /// Check if confidence meets minimum threshold
    pub fn meets_threshold(&self, threshold: Decimal) -> bool {
        self.overall >= threshold
    }

    /// Get confidence tier
    pub fn tier(&self) -> ConfidenceTier {
        if self.overall >= Decimal::from_f64(0.95).unwrap_or(Decimal::ONE) {
            ConfidenceTier::Exceptional
        } else if self.overall >= Decimal::from_f64(0.85).unwrap_or(Decimal::ONE) {
            ConfidenceTier::High
        } else if self.overall >= Decimal::from_f64(0.70).unwrap_or(Decimal::ONE) {
            ConfidenceTier::Good
        } else if self.overall >= Decimal::from_f64(0.55).unwrap_or(Decimal::ONE) {
            ConfidenceTier::Moderate
        } else if self.overall >= Decimal::from_f64(0.40).unwrap_or(Decimal::ONE) {
            ConfidenceTier::Low
        } else {
            ConfidenceTier::VeryLow
        }
    }

    /// Get risk multiplier based on confidence
    pub fn risk_multiplier(&self) -> Decimal {
        match self.tier() {
            ConfidenceTier::Exceptional => Decimal::from_f64(1.5).unwrap(),
            ConfidenceTier::High => Decimal::from_f64(1.25).unwrap(),
            ConfidenceTier::Good => Decimal::ONE,
            ConfidenceTier::Moderate => Decimal::from_f64(0.75).unwrap(),
            ConfidenceTier::Low => Decimal::from_f64(0.5).unwrap(),
            ConfidenceTier::VeryLow => Decimal::from_f64(0.25).unwrap(),
        }
    }
}

impl Default for ConfidenceScore {
    fn default() -> Self {
        Self {
            overall: Decimal::from_f64(0.5).unwrap(),
            signal: Decimal::from_f64(0.5).unwrap(),
            confluence: Decimal::from_f64(0.5).unwrap(),
            regime: Decimal::from_f64(0.5).unwrap(),
            market_condition_quality: Decimal::from_f64(0.5).unwrap(),
        }
    }
}

/// Confidence quality tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConfidenceTier {
    /// < 40%
    VeryLow = 1,
    /// 40-55%
    Low = 2,
    /// 55-70%
    Moderate = 3,
    /// 70-85%
    Good = 4,
    /// 85-95%
    High = 5,
    /// > 95%
    Exceptional = 6,
}

/// Confidence calculation engine
pub struct ConfidenceEngine {
    /// Weight for signal confidence
    signal_weight: Decimal,
    /// Weight for confluence score
    confluence_weight: Decimal,
    /// Weight for regime quality
    regime_weight: Decimal,
    /// Weight for pattern quality
    pattern_weight: Decimal,
    /// Weight for session quality
    session_weight: Decimal,
}

impl ConfidenceEngine {
    /// Create new confidence engine with default weights
    pub fn new() -> Self {
        Self {
            signal_weight: Decimal::from_f64(0.35).unwrap(),
            confluence_weight: Decimal::from_f64(0.25).unwrap(),
            regime_weight: Decimal::from_f64(0.20).unwrap(),
            pattern_weight: Decimal::from_f64(0.15).unwrap(),
            session_weight: Decimal::from_f64(0.05).unwrap(),
        }
    }

    /// Calculate overall confidence score
    pub fn calculate(&self, inputs: &ConfidenceInputs) -> ConfidenceScore {
        // Normalize confluence from 0-10 to 0-1
        let normalized_confluence = (inputs.confluence_score / Decimal::from(10))
            .clamp(Decimal::ZERO, Decimal::ONE);

        // Calculate weighted overall confidence
        let overall = (inputs.signal_confidence * self.signal_weight)
            + (normalized_confluence * self.confluence_weight)
            + (inputs.regime_quality * self.regime_weight)
            + (inputs.pattern_quality * self.pattern_weight)
            + (inputs.session_quality * self.session_weight);

        // Market condition quality is average of regime and session
        let market_condition_quality =
            (inputs.regime_quality + inputs.session_quality) / Decimal::from(2);

        ConfidenceScore {
            overall: overall.clamp(Decimal::ZERO, Decimal::ONE),
            signal: inputs.signal_confidence,
            confluence: normalized_confluence,
            regime: inputs.regime_quality,
            market_condition_quality,
        }
    }

    /// Check if inputs are sufficient for high confidence trading
    pub fn is_tradeable(&self, score: &ConfidenceScore, min_threshold: Decimal) -> bool {
        score.meets_threshold(min_threshold)
    }

    /// Get adjusted position size multiplier
    pub fn size_multiplier(&self, score: &ConfidenceScore) -> Decimal {
        score.risk_multiplier()
    }

    /// Explain the confidence calculation
    pub fn explain(&self, inputs: &ConfidenceInputs, score: &ConfidenceScore) -> String {
        format!(
            "Confidence: {} = \
             signal({:.0}%) * {:.0}% + \
             confluence({:.0}%) * {:.0}% + \
             regime({:.0}%) * {:.0}% + \
             pattern({:.0}%) * {:.0}% + \
             session({:.0}%) * {:.0}%",
            score.overall * Decimal::from(100),
            inputs.signal_confidence * Decimal::from(100),
            self.signal_weight * Decimal::from(100),
            (inputs.confluence_score / Decimal::from(10)) * Decimal::from(100),
            self.confluence_weight * Decimal::from(100),
            inputs.regime_quality * Decimal::from(100),
            self.regime_weight * Decimal::from(100),
            inputs.pattern_quality * Decimal::from(100),
            self.pattern_weight * Decimal::from(100),
            inputs.session_quality * Decimal::from(100),
            self.session_weight * Decimal::from(100),
        )
    }
}

impl Default for ConfidenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_inputs() -> ConfidenceInputs {
        ConfidenceInputs {
            signal_confidence: Decimal::from_f64(0.8).unwrap(),
            confluence_score: Decimal::from(7),
            regime_quality: Decimal::from_f64(0.7).unwrap(),
            pattern_quality: Decimal::from_f64(0.75).unwrap(),
            session_quality: Decimal::from_f64(0.8).unwrap(),
        }
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = ConfidenceEngine::new();
        let inputs = test_inputs();

        let score = engine.calculate(&inputs);

        assert!(score.overall > Decimal::ZERO);
        assert!(score.overall <= Decimal::ONE);
    }

    #[test]
    fn test_confidence_tiers() {
        let high_score = ConfidenceScore {
            overall: Decimal::from_f64(0.90).unwrap(),
            ..Default::default()
        };

        assert_eq!(high_score.tier(), ConfidenceTier::High);
        assert!(high_score.risk_multiplier() > Decimal::ONE);

        let low_score = ConfidenceScore {
            overall: Decimal::from_f64(0.30).unwrap(),
            ..Default::default()
        };

        assert_eq!(low_score.tier(), ConfidenceTier::VeryLow);
    }

    #[test]
    fn test_tradeability_check() {
        let engine = ConfidenceEngine::new();
        let score = ConfidenceScore {
            overall: Decimal::from_f64(0.8).unwrap(),
            ..Default::default()
        };

        assert!(engine.is_tradeable(&score, Decimal::from_f64(0.7).unwrap()));
        assert!(!engine.is_tradeable(&score, Decimal::from_f64(0.9).unwrap()));
    }

    #[test]
    fn test_confluence_normalization() {
        let engine = ConfidenceEngine::new();
        let inputs = ConfidenceInputs {
            confluence_score: Decimal::from(8),
            ..test_inputs()
        };

        let score = engine.calculate(&inputs);
        assert_eq!(score.confluence, Decimal::from_f64(0.8).unwrap());
    }
}
