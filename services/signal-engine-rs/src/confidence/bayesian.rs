//! Bayesian Confidence Updater
//!
//! Implements Beta-distribution Bayesian updating for per-strategy confidence.
//! After every completed trade, updates α (wins) and β (losses).
//!
//! Posterior mean: α / (α + β)
//! Applied shrinkage toward 0.50 prior when n < 30 (James-Stein style).
//!
//! This module is consumed by the learning engine which calls `update()` after
//! each completed trade and `posterior_confidence()` when scoring signals.

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Beta distribution parameters for a strategy.
#[derive(Debug, Clone)]
pub struct BetaParams {
    /// Prior α: starts at 1 (1 pseudo-win = uninformative)
    pub alpha: f64,
    /// Prior β: starts at 1 (1 pseudo-loss = uninformative)
    pub beta: f64,
    /// Total real sample count (excluding pseudo-observations)
    pub sample_count: u64,
}

impl Default for BetaParams {
    fn default() -> Self {
        Self {
            alpha: 1.0, // Jeffreys prior
            beta: 1.0,
            sample_count: 0,
        }
    }
}

impl BetaParams {
    /// Posterior mean: α / (α + β)
    pub fn posterior_mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Posterior variance: αβ / [(α+β)²(α+β+1)]
    pub fn posterior_variance(&self) -> f64 {
        let n = self.alpha + self.beta;
        (self.alpha * self.beta) / (n * n * (n + 1.0))
    }

    /// 95% credible interval half-width (approximate, normal approximation of Beta)
    pub fn uncertainty_95pct(&self) -> f64 {
        let var = self.posterior_variance();
        1.96 * var.sqrt()
    }
}

/// Bayesian confidence updater maintaining per-strategy Beta posteriors.
///
/// Thread-safe via external locking (caller holds lock or uses DashMap).
pub struct BayesianConfidenceUpdater {
    /// Per-strategy Beta parameters
    posteriors: HashMap<String, BetaParams>,
    /// Shrinkage threshold: below this sample count, blend toward 0.50
    shrinkage_threshold: u64,
    /// Regime quality multiplier (0.5–1.5): from market regime detector
    regime_quality: f64,
    /// Execution quality multiplier (0.5–1.0): penalise high slippage
    execution_quality: f64,
}

impl BayesianConfidenceUpdater {
    pub fn new() -> Self {
        Self {
            posteriors: HashMap::new(),
            shrinkage_threshold: 30,
            regime_quality: 1.0,
            execution_quality: 1.0,
        }
    }

    /// Update posterior for a strategy after a completed trade.
    ///
    /// Call this recursively after every completed trade (from learning engine feed).
    pub fn update(&mut self, strategy_id: &str, trade_was_winner: bool) {
        let entry = self.posteriors.entry(strategy_id.to_string()).or_default();
        if trade_was_winner {
            entry.alpha += 1.0;
        } else {
            entry.beta += 1.0;
        }
        entry.sample_count += 1;
    }

    /// Get posterior confidence for a strategy (0–1 Decimal).
    ///
    /// Applies:
    /// 1. James-Stein shrinkage toward 0.50 when n < threshold
    /// 2. Regime quality adjustment
    /// 3. Execution quality adjustment
    pub fn posterior_confidence(&self, strategy_id: &str) -> Decimal {
        let params = match self.posteriors.get(strategy_id) {
            Some(p) => p,
            None => return f64_to_decimal(0.5), // No data → maximum uncertainty
        };

        let raw_mean = params.posterior_mean();

        // James-Stein shrinkage: blend toward 0.50 for small samples
        let shrunk = if params.sample_count < self.shrinkage_threshold {
            let shrinkage_factor = params.sample_count as f64 / self.shrinkage_threshold as f64;
            raw_mean * shrinkage_factor + 0.5 * (1.0 - shrinkage_factor)
        } else {
            raw_mean
        };

        // Regime and execution quality adjustments
        let adjusted = (shrunk * self.regime_quality * self.execution_quality).clamp(0.0, 1.0);

        f64_to_decimal(adjusted)
    }

    /// Update regime quality factor from market regime detector.
    ///
    /// regime_quality ∈ [0.5, 1.5]: 1.0 = neutral, >1.0 = favourable, <1.0 = adverse
    pub fn set_regime_quality(&mut self, quality: f64) {
        self.regime_quality = quality.clamp(0.5, 1.5);
    }

    /// Update execution quality from recent slippage data.
    ///
    /// execution_quality ∈ [0.5, 1.0]: 1.0 = perfect fill, 0.5 = extreme slippage
    pub fn set_execution_quality(&mut self, quality: f64) {
        self.execution_quality = quality.clamp(0.5, 1.0);
    }

    /// Get Beta parameters for a strategy (useful for monitoring / persistence).
    pub fn get_params(&self, strategy_id: &str) -> Option<&BetaParams> {
        self.posteriors.get(strategy_id)
    }

    /// Uncertainty width (95% CI half-width) for a strategy.
    /// Higher = less confident in the estimate.
    pub fn uncertainty(&self, strategy_id: &str) -> Decimal {
        let u = self
            .posteriors
            .get(strategy_id)
            .map(|p| p.uncertainty_95pct())
            .unwrap_or(0.25); // Maximum uncertainty for unknown strategy
        f64_to_decimal(u)
    }

    /// Number of strategies being tracked.
    pub fn tracked_strategies(&self) -> usize {
        self.posteriors.len()
    }
}

impl Default for BayesianConfidenceUpdater {
    fn default() -> Self {
        Self::new()
    }
}

fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::from_f64(v).unwrap_or(Decimal::ZERO).round_dp(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posterior_converges_to_win_rate() {
        let mut updater = BayesianConfidenceUpdater::new();
        // Feed 100 trades with 60% win rate
        for i in 0..100 {
            updater.update("strat_a", i % 10 < 6); // 6/10 wins
        }
        let conf = updater.posterior_confidence("strat_a");
        let conf_f = conf.to_f64().unwrap_or(0.0);
        // With 100 samples, posterior should be close to 0.60 (within 0.05)
        assert!(
            (conf_f - 0.60).abs() < 0.05,
            "Expected ~0.60, got {}",
            conf_f
        );
    }

    #[test]
    fn test_shrinkage_with_small_sample() {
        let mut updater = BayesianConfidenceUpdater::new();
        // Only 5 trades, all wins
        for _ in 0..5 {
            updater.update("strat_b", true);
        }
        let conf = updater.posterior_confidence("strat_b");
        let conf_f = conf.to_f64().unwrap_or(0.0);
        // Should be shrunk toward 0.5 (5/30 shrinkage factor)
        // Without shrinkage it would be (1+5)/(1+5+1) ≈ 0.857
        // With shrinkage: 0.857 × (5/30) + 0.5 × (25/30) ≈ 0.143 + 0.417 = 0.56
        assert!(conf_f < 0.75, "Should be shrunk toward 0.5, got {}", conf_f);
    }

    #[test]
    fn test_unknown_strategy_returns_half() {
        let updater = BayesianConfidenceUpdater::new();
        let conf = updater.posterior_confidence("unknown_strat");
        assert_eq!(conf, f64_to_decimal(0.5));
    }

    #[test]
    fn test_regime_quality_scales_confidence() {
        let mut updater = BayesianConfidenceUpdater::new();
        for _ in 0..50 {
            updater.update("strat_c", true);
        }
        let base = updater.posterior_confidence("strat_c");
        updater.set_regime_quality(0.7); // Adverse regime
        let adverse = updater.posterior_confidence("strat_c");
        assert!(adverse < base, "Adverse regime should reduce confidence");
    }
}
