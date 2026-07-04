//! Learning Engine Confidence Module — Bayesian per-strategy confidence scoring.
//!
//! Replaces the simple arithmetic average with:
//! 1. Bayesian Beta posterior for win-rate confidence
//! 2. Sample-size-weighted confidence (Wilson score interval)
//! 3. Regime-adjusted confidence (multiples applied to posterior)
//! 4. Execution-adjusted confidence (penalise high slippage)
//!
//! Produces a composite confidence score in [0, 100].

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use serde::{Deserialize, Serialize};

/// Input metrics for confidence computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    /// Total trades completed (must be ≥ 0)
    pub total_trades: u64,
    /// Number of winning trades
    pub winning_trades: u64,
    /// Regime quality multiplier from regime detector (0.5 = adverse, 1.0 = neutral, 1.5 = favourable)
    pub regime_quality: Decimal,
    /// Execution quality multiplier (0.5 = high slippage, 1.0 = perfect fill)
    pub execution_quality: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceOutput {
    /// Bayesian posterior win-rate (mean of Beta posterior)
    pub bayesian_win_rate: Decimal,
    /// Wilson score lower bound (95% CI lower)
    pub wilson_lower_bound: Decimal,
    /// Final composite confidence [0, 100]
    pub composite_score: u8,
    /// Sample size adequacy (1.0 when n ≥ 100, scales linearly below)
    pub sample_adequacy: Decimal,
}

pub struct ConfidenceEngine;

impl Default for ConfidenceEngine {
    fn default() -> Self { Self::new() }
}

impl ConfidenceEngine {
    pub fn new() -> Self { Self }

    /// Compute Bayesian confidence for a strategy.
    ///
    /// Beta(α, β) where α = wins + 1, β = losses + 1 (Laplace prior).
    /// Wilson score gives stable CI for small samples.
    pub fn compute_confidence(&self, metrics: &ConfidenceMetrics) -> ConfidenceOutput {
        let wins = metrics.winning_trades as f64;
        let n = metrics.total_trades as f64;
        let losses = n - wins;

        // Bayesian posterior: Beta(α=wins+1, β=losses+1)
        let alpha = wins + 1.0;
        let beta_param = losses + 1.0;
        let posterior_mean = alpha / (alpha + beta_param);

        // Wilson score lower bound (95% CI, z = 1.96)
        let wilson_lower = if n > 0.0 {
            let p_hat = wins / n;
            let z = 1.96_f64;
            let z2 = z * z;
            let numerator = p_hat + z2 / (2.0 * n) - z * (p_hat * (1.0 - p_hat) / n + z2 / (4.0 * n * n)).sqrt();
            let denominator = 1.0 + z2 / n;
            (numerator / denominator).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Sample adequacy: ramp from 0 to 1 as n goes from 0 to 100
        let sample_adequacy = (n / 100.0).clamp(0.0, 1.0);

        // Composite: (wilson_lower × 0.6 + posterior_mean × 0.4) × sample_adequacy
        // × regime_quality × execution_quality
        let regime_f = metrics.regime_quality.to_f64().unwrap_or(1.0).clamp(0.5, 1.5);
        let exec_f = metrics.execution_quality.to_f64().unwrap_or(1.0).clamp(0.5, 1.0);

        let base = wilson_lower * 0.6 + posterior_mean * 0.4;
        let adjusted = (base * sample_adequacy * regime_f * exec_f).clamp(0.0, 1.0);
        let composite_score = (adjusted * 100.0).round() as u8;

        ConfidenceOutput {
            bayesian_win_rate: Decimal::from_f64(posterior_mean)
                .unwrap_or(Decimal::new(5, 1))
                .round_dp(4),
            wilson_lower_bound: Decimal::from_f64(wilson_lower)
                .unwrap_or(Decimal::ZERO)
                .round_dp(4),
            composite_score,
            sample_adequacy: Decimal::from_f64(sample_adequacy)
                .unwrap_or(Decimal::ZERO)
                .round_dp(4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_win_rate_high_score() {
        let engine = ConfidenceEngine::new();
        let metrics = ConfidenceMetrics {
            total_trades: 200,
            winning_trades: 140, // 70% win rate
            regime_quality: Decimal::ONE,
            execution_quality: Decimal::ONE,
        };
        let out = engine.compute_confidence(&metrics);
        assert!(out.composite_score > 50, "70% WR at n=200 should score > 50, got {}", out.composite_score);
    }

    #[test]
    fn test_zero_trades_returns_low_score() {
        let engine = ConfidenceEngine::new();
        let metrics = ConfidenceMetrics {
            total_trades: 0,
            winning_trades: 0,
            regime_quality: Decimal::ONE,
            execution_quality: Decimal::ONE,
        };
        let out = engine.compute_confidence(&metrics);
        assert_eq!(out.composite_score, 0);
    }

    #[test]
    fn test_adverse_regime_reduces_score() {
        let engine = ConfidenceEngine::new();
        let base = ConfidenceMetrics {
            total_trades: 100,
            winning_trades: 60,
            regime_quality: Decimal::ONE,
            execution_quality: Decimal::ONE,
        };
        let adverse = ConfidenceMetrics {
            regime_quality: Decimal::new(5, 1), // 0.5
            ..base.clone()
        };
        let base_score = engine.compute_confidence(&base).composite_score;
        let adverse_score = engine.compute_confidence(&adverse).composite_score;
        assert!(adverse_score < base_score, "Adverse regime should reduce score");
    }
}
