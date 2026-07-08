//! Confidence calculator
//!
//! Generates calibrated confidence scores based on multiple quality dimensions.
//! Confidence is designed to be well-calibrated - a 70% confidence should
//! correspond to a ~70% win rate historically.

use crate::confidence::decay::ConfidenceDecay;
use crate::confluence::ConfluenceScore;
use crate::mtf::types::MTFAlignmentResult;
use crate::regime::MarketRegime;
use crate::signals::result::SignalDirection;
use crate::smc::SMCAnalysis;
use crate::structure::StructureAnalysis;

/// Confidence calculation result
#[derive(Debug, Clone)]
pub struct ConfidenceResult {
    /// Overall confidence (0-100)
    pub overall: f64,
    /// Pattern quality contribution
    pub pattern_quality: f64,
    /// Structure quality contribution
    pub structure_quality: f64,
    /// Regime quality contribution
    pub regime_quality: f64,
    /// MTF agreement contribution
    pub mtf_agreement: f64,
    /// Volatility adjustment
    pub volatility_adjustment: f64,
    /// Time decay adjustment
    pub time_decay: f64,
    /// Calibration factor applied
    pub calibration_factor: f64,
}

impl ConfidenceResult {
    /// Check if confidence is high enough for trading
    pub fn is_tradable(&self, threshold: f64) -> bool {
        self.overall >= threshold
    }

    /// Get confidence as percentage string
    pub fn as_percentage(&self) -> String {
        format!("{:.1}%", self.overall)
    }

    /// Get confidence tier
    pub fn tier(&self) -> ConfidenceTier {
        match self.overall {
            c if c >= 80.0 => ConfidenceTier::High,
            c if c >= 60.0 => ConfidenceTier::Medium,
            c if c >= 40.0 => ConfidenceTier::Low,
            _ => ConfidenceTier::VeryLow,
        }
    }
}

/// Confidence quality tiers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfidenceTier {
    /// Very low confidence (< 40%)
    VeryLow,
    /// Low confidence (40-60%)
    Low,
    /// Medium confidence (60-80%)
    Medium,
    /// High confidence (> 80%)
    High,
}

/// Calculates calibrated confidence scores
#[derive(Debug)]
pub struct ConfidenceCalculator {
    /// Base calibration factor
    calibration: f64,
    /// Decay model
    decay: ConfidenceDecay,
}

impl ConfidenceCalculator {
    /// Create new confidence calculator
    pub fn new() -> Self {
        Self {
            calibration: 0.95, // Slight underconfidence to start
            decay: ConfidenceDecay::default(),
        }
    }

    /// Calculate confidence for a signal
    pub fn calculate(
        &self,
        direction: SignalDirection,
        confluence: &ConfluenceScore,
        structure: &StructureAnalysis,
        mtf: &MTFAlignmentResult,
        regime: &MarketRegime,
        smc: &SMCAnalysis,
    ) -> ConfidenceResult {
        // 1. Pattern quality score
        let pattern_quality = self.calculate_pattern_quality(smc);

        // 2. Structure quality score
        let structure_quality = self.calculate_structure_quality(structure);

        // 3. Regime quality score
        let regime_quality = self.calculate_regime_quality(regime, direction);

        // 4. MTF agreement score
        let mtf_agreement = self.calculate_mtf_agreement(mtf, direction);

        // 5. Volatility adjustment
        let volatility_adjustment = self.calculate_volatility_adjustment(regime);

        // 6. Time decay (signals degrade over time)
        let time_decay = self.decay.current_decay();

        // Combine with confluence
        let confluence_factor = confluence.total as f64 / 100.0;

        // Weighted combination
        let weighted_sum = pattern_quality * 0.25
            + structure_quality * 0.20
            + regime_quality * 0.15
            + mtf_agreement * 0.20
            + confluence_factor * 0.20;

        // Apply adjustments
        let adjusted = weighted_sum * volatility_adjustment * time_decay;

        // Apply calibration
        let calibrated = adjusted * self.calibration;

        // Bound to 0-100
        let overall = calibrated.clamp(0.0, 100.0);

        ConfidenceResult {
            overall,
            pattern_quality: pattern_quality * 100.0,
            structure_quality: structure_quality * 100.0,
            regime_quality: regime_quality * 100.0,
            mtf_agreement: mtf_agreement * 100.0,
            volatility_adjustment,
            time_decay,
            calibration_factor: self.calibration,
        }
    }

    fn calculate_pattern_quality(&self, smc: &SMCAnalysis) -> f64 {
        let mut score = 0.5; // Neutral base

        // SMC pattern quality
        let ob_quality = smc
            .freshest_bullish_ob()
            .or_else(|| smc.freshest_bearish_ob())
            .map(|ob| ob.strength)
            .unwrap_or(0.0);

        let fvg_quality = smc
            .fresh_fvgs(20)
            .iter()
            .map(|f| f.strength)
            .fold(0.0_f64, |a, b| a.max(b));

        let liquidity_quality = smc
            .liquidity
            .strongest_sweep
            .as_ref()
            .map(|s| s.strength)
            .unwrap_or(0.0);

        let displacement_quality = if !smc.displacements.is_empty() {
            let recent = &smc.displacements[smc.displacements.len() - 1];
            recent.strength
        } else {
            0.0
        };

        // Weight factors
        score += ob_quality * 0.3;
        score += fvg_quality * 0.2;
        score += liquidity_quality * 0.2;
        score += displacement_quality * 0.15;

        // Penalize if imbalance contradicts signal
        if smc.imbalance.dominant_bias != crate::smc::ImbalanceDirection::Neutral {
            // Currently not checking direction TODO: improve
        }

        score.min(1.0)
    }

    fn calculate_structure_quality(&self, structure: &StructureAnalysis) -> f64 {
        let swing_count = structure.swing_highs.len().min(structure.swing_lows.len());

        let clarity = match swing_count {
            0 => 0.2,
            1 => 0.4,
            2 => 0.55,
            3 => 0.7,
            4..=6 => 0.85,
            _ => 0.95,
        };

        // Bonus for clear range structure
        let range_bonus = if structure.range.is_some() { 0.05 } else { 0.0 };

        f64::min(clarity + range_bonus, 1.0_f64)
    }

    fn calculate_regime_quality(&self, regime: &MarketRegime, direction: SignalDirection) -> f64 {
        use crate::regime::RegimeType;

        let base_quality = regime.confidence;

        // Regime direction alignment
        let alignment_bonus = match (regime.regime_type, direction) {
            (RegimeType::TrendingUp, SignalDirection::Long) => 0.2,
            (RegimeType::TrendingDown, SignalDirection::Short) => 0.2,
            (RegimeType::HighVolatility, _) => -0.3,
            (RegimeType::Undefined, _) => -0.4,
            (RegimeType::Transition, _) => -0.2,
            _ => 0.0,
        };

        (base_quality + alignment_bonus).clamp(0.0, 1.0)
    }

    fn calculate_mtf_agreement(&self, mtf: &MTFAlignmentResult, direction: SignalDirection) -> f64 {
        use crate::mtf::types::{AlignmentDirection, MarketBias};

        let alignment_score = mtf.alignment_score;

        // Check directional agreement
        let direction_aligned = match (mtf.bias, direction) {
            (MarketBias::StrongBullish | MarketBias::Bullish, SignalDirection::Long) => true,
            (MarketBias::StrongBearish | MarketBias::Bearish, SignalDirection::Short) => true,
            _ => false,
        };

        if !direction_aligned {
            return alignment_score * 0.3;
        }

        // Count agreeing timeframes
        let agreeing = mtf
            .alignments
            .iter()
            .filter(|a| {
                matches!(
                    (a.direction, direction),
                    (AlignmentDirection::Bullish, SignalDirection::Long)
                        | (AlignmentDirection::Bearish, SignalDirection::Short)
                )
            })
            .count();

        let tf_bonus = (agreeing as f64) * 0.1;

        (alignment_score + tf_bonus).min(1.0)
    }

    fn calculate_volatility_adjustment(&self, regime: &MarketRegime) -> f64 {
        // Optimal volatility is middle range
        let vol_pct = regime.volatility_percentile;

        match vol_pct {
            v if v < 0.1 => 0.9, // Too quiet
            v if v < 0.3 => 1.0, // Good
            v if v < 0.7 => 1.0, // Optimal
            v if v < 0.9 => 0.9, // Elevated
            _ => 0.7,            // Too volatile
        }
    }

    /// Update calibration based on actual results (Bayesian update using Beta distribution)
    pub fn update_calibration(&mut self, predicted: f64, actual: bool) {
        // We use a Beta distribution (alpha, beta) for Bayesian update
        // We can approximate the current calibration state
        // Let's assume an effective sample size (ESS) of 100 for smoothing
        let ess = 100.0;
        let mut alpha = self.calibration * ess;
        let mut beta_param = ess - alpha;

        // Update priors
        if actual {
            alpha += 1.0;
        } else {
            beta_param += 1.0;
        }

        // Calculate new expected value (posterior mean)
        let expected_win_rate = alpha / (alpha + beta_param);

        // Ratio of actual win rate vs predicted is our new calibration
        let prediction_decimal = (predicted / 100.0).clamp(0.01, 0.99);
        let new_calibration = expected_win_rate / prediction_decimal;

        // Exponential smoothing to avoid sudden jumps
        let learning_rate = 0.05;
        self.calibration = (self.calibration * (1.0 - learning_rate)
            + new_calibration * learning_rate)
            .clamp(0.5, 1.2);
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_bounds() {
        let calc = ConfidenceCalculator::new();

        // Test that confidence stays within bounds
        let result = calc.calculate(
            SignalDirection::Long,
            &crate::confluence::ConfluenceScore {
                total: 50,
                factors: vec![],
            },
            &crate::structure::StructureAnalysis {
                swing_highs: vec![],
                swing_lows: vec![],
                trend: crate::structure::trend::TrendDirection::Undefined,
                range: None,
            },
            &MTFAlignmentResult {
                aligned: false,
                reference_timeframe: "H4".to_string(),
                alignments: vec![],
                alignment_score: 0.5,
                bias: crate::mtf::types::MarketBias::Neutral,
            },
            &MarketRegime {
                regime_type: crate::regime::RegimeType::TrendingUp,
                confidence: 0.7,
                volatility_percentile: 0.5,
                trend_strength: 0.6,
            },
            &SMCAnalysis::empty(),
        );

        assert!(result.overall >= 0.0 && result.overall <= 100.0);
    }

    #[test]
    fn test_confidence_tier() {
        let result = ConfidenceResult {
            overall: 75.0,
            pattern_quality: 70.0,
            structure_quality: 70.0,
            regime_quality: 70.0,
            mtf_agreement: 70.0,
            volatility_adjustment: 1.0,
            time_decay: 1.0,
            calibration_factor: 0.95,
        };

        assert_eq!(result.tier(), ConfidenceTier::Medium);
    }
}
