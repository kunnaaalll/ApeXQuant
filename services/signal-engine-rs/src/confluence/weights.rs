//! Dynamic weight adjustment for confluence factors

use crate::confluence::factors::FactorType;
use crate::regime::RegimeType;
use std::collections::HashMap;

/// Adjusts factor weights based on market conditions
#[derive(Debug)]
pub struct WeightAdjuster {
    base_weights: HashMap<FactorType, f64>,
    regime_adjustments: HashMap<RegimeType, HashMap<FactorType, f64>>,
}

impl WeightAdjuster {
    /// Create new weight adjuster with default settings
    pub fn new() -> Self {
        let mut base_weights = HashMap::new();

        // Set base weights
        for factor in [
            FactorType::TimeframeAlignment,
            FactorType::TrendQuality,
            FactorType::Regime,
            FactorType::OrderBlock,
            FactorType::FairValueGap,
            FactorType::Liquidity,
            FactorType::Displacement,
            FactorType::Momentum,
            FactorType::Volatility,
            FactorType::Session,
            FactorType::Structure,
            FactorType::RiskReward,
        ] {
            base_weights.insert(factor, factor.default_weight());
        }

        let regime_adjustments = Self::build_regime_adjustments();

        Self {
            base_weights,
            regime_adjustments,
        }
    }

    /// Get weight for a factor in current regime
    pub fn get_weight(&self, factor: FactorType, regime: RegimeType) -> f64 {
        let base = self.base_weights.get(&factor).copied().unwrap_or(1.0);

        let adjustment = self
            .regime_adjustments
            .get(&regime)
            .and_then(|m| m.get(&factor))
            .copied()
            .unwrap_or(1.0);

        base * adjustment
    }

    /// Get all weights for a regime
    pub fn get_weights_for_regime(&self, regime: RegimeType) -> HashMap<FactorType, f64> {
        let mut weights = HashMap::new();

        for factor in self.base_weights.keys() {
            weights.insert(*factor, self.get_weight(*factor, regime));
        }

        weights
    }

    /// Build regime-specific adjustments
    fn build_regime_adjustments() -> HashMap<RegimeType, HashMap<FactorType, f64>> {
        let mut adjustments = HashMap::new();

        // Trending Up - emphasize trend, reduce reversal signals
        let mut trending_up = HashMap::new();
        trending_up.insert(FactorType::TrendQuality, 1.3);
        trending_up.insert(FactorType::Displacement, 1.2);
        trending_up.insert(FactorType::Momentum, 1.1);
        trending_up.insert(FactorType::Liquidity, 0.9);
        adjustments.insert(RegimeType::TrendingUp, trending_up);

        // Trending Down - emphasize trend, reduce reversal signals
        let mut trending_down = HashMap::new();
        trending_down.insert(FactorType::TrendQuality, 1.3);
        trending_down.insert(FactorType::Displacement, 1.2);
        trending_down.insert(FactorType::Momentum, 1.1);
        trending_down.insert(FactorType::Liquidity, 0.9);
        adjustments.insert(RegimeType::TrendingDown, trending_down);

        // Ranging - emphasize mean reversion, reduce trend following
        let mut ranging = HashMap::new();
        ranging.insert(FactorType::OrderBlock, 1.2);
        ranging.insert(FactorType::FairValueGap, 1.1);
        ranging.insert(FactorType::RiskReward, 1.2);
        ranging.insert(FactorType::TrendQuality, 0.6);
        ranging.insert(FactorType::Displacement, 0.8);
        adjustments.insert(RegimeType::Ranging, ranging);

        // Low Volatility - emphasize breakout patterns
        let mut low_vol = HashMap::new();
        low_vol.insert(FactorType::Displacement, 1.3);
        low_vol.insert(FactorType::OrderBlock, 1.1);
        low_vol.insert(FactorType::Volatility, 1.2);
        adjustments.insert(RegimeType::LowVolatility, low_vol);

        // High Volatility - reduce size, emphasize structure
        let mut high_vol = HashMap::new();
        high_vol.insert(FactorType::Structure, 1.2);
        high_vol.insert(FactorType::OrderBlock, 1.1);
        high_vol.insert(FactorType::Volatility, 1.2);
        high_vol.insert(FactorType::TrendQuality, 0.8);
        adjustments.insert(RegimeType::HighVolatility, high_vol);

        // Breakout - emphasize displacement and volume
        let mut breakout = HashMap::new();
        breakout.insert(FactorType::Displacement, 1.3);
        breakout.insert(FactorType::Momentum, 1.2);
        breakout.insert(FactorType::Liquidity, 1.1);
        adjustments.insert(RegimeType::Breakout, breakout);

        // Transition - conservative, emphasize structure
        let mut transition = HashMap::new();
        transition.insert(FactorType::Structure, 1.2);
        transition.insert(FactorType::TimeframeAlignment, 1.1);
        transition.insert(FactorType::Displacement, 0.8);
        transition.insert(FactorType::Momentum, 0.8);
        adjustments.insert(RegimeType::Transition, transition);

        adjustments
    }
}

impl Default for WeightAdjuster {
    fn default() -> Self {
        Self::new()
    }
}

/// Adaptive weight configuration
#[derive(Debug, Clone)]
pub struct AdaptiveWeights {
    /// Current regime
    pub regime: RegimeType,
    /// Active weights
    pub weights: HashMap<FactorType, f64>,
    /// Last updated
    pub last_updated: time::OffsetDateTime,
}

impl AdaptiveWeights {
    /// Create new adaptive weights
    pub fn new(regime: RegimeType) -> Self {
        let adjuster = WeightAdjuster::new();
        let weights = adjuster.get_weights_for_regime(regime);

        Self {
            regime,
            weights,
            last_updated: time::OffsetDateTime::now_utc(),
        }
    }

    /// Update for new regime
    pub fn update_regime(&mut self, new_regime: RegimeType) {
        if new_regime != self.regime {
            let adjuster = WeightAdjuster::new();
            self.weights = adjuster.get_weights_for_regime(new_regime);
            self.regime = new_regime;
            self.last_updated = time::OffsetDateTime::now_utc();
        }
    }

    /// Get weight for factor
    pub fn get(&self, factor: FactorType) -> f64 {
        self.weights.get(&factor).copied().unwrap_or(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_adjuster_trending_up() {
        let adjuster = WeightAdjuster::new();

        let trend_weight = adjuster.get_weight(FactorType::TrendQuality, RegimeType::TrendingUp);
        let base_weight = FactorType::TrendQuality.default_weight();

        assert!(trend_weight > base_weight);
    }

    #[test]
    fn test_weight_adjuster_ranging() {
        let adjuster = WeightAdjuster::new();

        let trend_weight = adjuster.get_weight(FactorType::TrendQuality, RegimeType::Ranging);
        let base_weight = FactorType::TrendQuality.default_weight();

        assert!(trend_weight < base_weight);
    }

    #[test]
    fn test_adaptive_weights() {
        let mut adaptive = AdaptiveWeights::new(RegimeType::TrendingUp);

        let initial_regime = adaptive.regime;
        adaptive.update_regime(RegimeType::Ranging);

        assert_ne!(initial_regime, adaptive.regime);
        assert!(adaptive.last_updated > time::OffsetDateTime::UNIX_EPOCH);
    }
}
