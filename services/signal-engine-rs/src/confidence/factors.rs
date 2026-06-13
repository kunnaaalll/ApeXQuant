//! Confidence factors
//!
//! Individual factors that contribute to confidence calculation.

/// Individual confidence factor
#[derive(Debug, Clone)]
pub struct ConfidenceFactor {
    /// Factor name
    pub name: String,
    /// Raw score (0-1)
    pub score: f64,
    /// Weight in overall calculation
    pub weight: f64,
    /// Description
    pub description: String,
}

/// Factor weights for confidence calculation
#[derive(Debug, Clone)]
pub struct FactorWeights {
    /// Pattern quality weight
    pub pattern_quality: f64,
    /// Structure quality weight
    pub structure_quality: f64,
    /// Regime alignment weight
    pub regime_alignment: f64,
    /// MTF agreement weight
    pub mtf_agreement: f64,
    /// Confluence weight
    pub confluence: f64,
}

impl FactorWeights {
    /// Create balanced weights
    pub fn balanced() -> Self {
        Self {
            pattern_quality: 0.25,
            structure_quality: 0.20,
            regime_alignment: 0.15,
            mtf_agreement: 0.20,
            confluence: 0.20,
        }
    }

    /// Create pattern-heavy weights
    pub fn pattern_focused() -> Self {
        Self {
            pattern_quality: 0.35,
            structure_quality: 0.15,
            regime_alignment: 0.10,
            mtf_agreement: 0.15,
            confluence: 0.25,
        }
    }

    /// Create structure-heavy weights
    pub fn structure_focused() -> Self {
        Self {
            pattern_quality: 0.15,
            structure_quality: 0.35,
            regime_alignment: 0.15,
            mtf_agreement: 0.20,
            confluence: 0.15,
        }
    }

    /// Verify weights sum to approximately 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.pattern_quality
            + self.structure_quality
            + self.regime_alignment
            + self.mtf_agreement
            + self.confluence;

        (sum - 1.0).abs() < 0.01
    }
}

impl Default for FactorWeights {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Pattern quality scoring
#[derive(Debug, Clone)]
pub struct PatternQuality {
    /// Order block score
    pub order_block: f64,
    /// FVG score
    pub fair_value_gap: f64,
    /// Liquidity sweep score
    pub liquidity_sweep: f64,
    /// Displacement score
    pub displacement: f64,
    /// Imbalance score
    pub imbalance: f64,
}

impl PatternQuality {
    /// Calculate overall pattern quality
    pub fn overall(&self) -> f64 {
        // Weighted average of pattern scores
        let weighted =
            self.order_block * 0.30 +
            self.fair_value_gap * 0.20 +
            self.liquidity_sweep * 0.20 +
            self.displacement * 0.20 +
            self.imbalance * 0.10;

        weighted.min(1.0)
    }

    /// Check if any pattern is strong
    pub fn has_strong_pattern(&self, threshold: f64) -> bool {
        self.order_block >= threshold
            || self.fair_value_gap >= threshold
            || self.liquidity_sweep >= threshold
            || self.displacement >= threshold
    }
}

impl Default for PatternQuality {
    fn default() -> Self {
        Self {
            order_block: 0.0,
            fair_value_gap: 0.0,
            liquidity_sweep: 0.0,
            displacement: 0.0,
            imbalance: 0.0,
        }
    }
}

/// Structure quality scoring
#[derive(Debug, Clone)]
pub struct StructureQuality {
    /// Swing clarity score
    pub swing_clarity: f64,
    /// Trend strength score
    pub trend_strength: f64,
    /// BOS/CHOCH presence
    pub structure_break: f64,
    /// Range quality (if ranging)
    pub range_quality: Option<f64>,
}

impl StructureQuality {
    /// Calculate overall structure quality
    pub fn overall(&self) -> f64 {
        let mut score = self.swing_clarity * 0.4 + self.trend_strength * 0.35 + self.structure_break * 0.25;

        // Range bonus if applicable
        if let Some(range_quality) = self.range_quality {
            score = score * 0.7 + range_quality * 0.3;
        }

        score.min(1.0)
    }
}

impl Default for StructureQuality {
    fn default() -> Self {
        Self {
            swing_clarity: 0.0,
            trend_strength: 0.0,
            structure_break: 0.0,
            range_quality: None,
        }
    }
}

/// Quality scoring configuration
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Minimum score for each component
    pub min_pattern_quality: f64,
    /// Minimum structure quality
    pub min_structure_quality: f64,
    /// Minimum regime alignment
    pub min_regime_alignment: f64,
    /// Minimum MTF agreement
    pub min_mtf_agreement: f64,
    /// Factor weights
    pub weights: FactorWeights,
}

impl QualityConfig {
    /// Create strict quality config
    pub fn strict() -> Self {
        Self {
            min_pattern_quality: 0.6,
            min_structure_quality: 0.6,
            min_regime_alignment: 0.5,
            min_mtf_agreement: 0.6,
            weights: FactorWeights::structure_focused(),
        }
    }

    /// Create lenient quality config
    pub fn lenient() -> Self {
        Self {
            min_pattern_quality: 0.3,
            min_structure_quality: 0.3,
            min_regime_alignment: 0.2,
            min_mtf_agreement: 0.3,
            weights: FactorWeights::pattern_focused(),
        }
    }

    /// Create default config (balanced)
    pub fn default() -> Self {
        Self {
            min_pattern_quality: 0.4,
            min_structure_quality: 0.4,
            min_regime_alignment: 0.3,
            min_mtf_agreement: 0.4,
            weights: FactorWeights::balanced(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factor_weights_sum() {
        let weights = FactorWeights::balanced();
        assert!(weights.is_valid());
    }

    #[test]
    fn test_pattern_quality_overall() {
        let quality = PatternQuality {
            order_block: 0.8,
            fair_value_gap: 0.6,
            liquidity_sweep: 0.7,
            displacement: 0.5,
            imbalance: 0.4,
        };

        let overall = quality.overall();
        assert!(overall > 0.0 && overall <= 1.0);
    }

    #[test]
    fn test_has_strong_pattern() {
        let quality = PatternQuality {
            order_block: 0.8,
            fair_value_gap: 0.0,
            liquidity_sweep: 0.0,
            displacement: 0.0,
            imbalance: 0.0,
        };

        assert!(quality.has_strong_pattern(0.7));
        assert!(!quality.has_strong_pattern(0.9));
    }
}
