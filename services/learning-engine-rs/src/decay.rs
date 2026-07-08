//! Learning Engine Decay Tracker — EMA-based strategy fitness decay.
//!
//! Replaces the static 1.5× urgency multiplier with:
//! - Exponential Moving Average (EMA) of per-trade returns
//! - Edge decay: EMA return falling below its own historical mean
//! - Confidence decay: shrinking sample adequacy (Wilson bound dropping)
//! - Regime decay: adverse regime quality × rolling regime changes
//! - Urgency: logistic function of composite decay, steeper near 1.0

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayMetrics {
    /// EMA of recent net PnL per trade (in account currency)
    pub ema_return: Decimal,
    /// Long-run mean return used as baseline (from full history)
    pub historical_mean_return: Decimal,
    /// Current win rate (Bayesian posterior mean)
    pub current_win_rate: Decimal,
    /// Baseline win rate from original training/certification
    pub baseline_win_rate: Decimal,
    /// Current regime quality factor (0.5 = adverse, 1.0 = neutral)
    pub regime_quality: Decimal,
    /// Number of regime transitions in last N periods
    pub regime_transitions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayOutput {
    /// Edge decay: how much the return edge has eroded (0 = none, 1 = total)
    pub edge_decay: Decimal,
    /// Confidence decay: how much win-rate confidence has dropped (0–1)
    pub confidence_decay: Decimal,
    /// Regime decay: instability from regime changes (0–1)
    pub regime_decay: Decimal,
    /// Composite decay score (0–1)
    pub decay_score: Decimal,
    /// Urgency to retrain: logistic function of decay_score (0–1)
    pub urgency_score: Decimal,
}

pub struct DecayTracker {
    /// EMA smoothing factor (α = 2/(N+1) for N-period EMA, default N=20)
    ema_alpha: f64,
}

impl Default for DecayTracker {
    fn default() -> Self { Self::new() }
}

impl DecayTracker {
    pub fn new() -> Self {
        Self { ema_alpha: 2.0 / 21.0 } // 20-period EMA
    }

    pub fn with_ema_period(n: u32) -> Self {
        Self { ema_alpha: 2.0 / (n as f64 + 1.0) }
    }

    /// Compute EMA-based decay scores.
    pub fn compute_decay(&self, metrics: &DecayMetrics) -> DecayOutput {
        let ema_f = metrics.ema_return.to_f64().unwrap_or(0.0);
        let hist_mean_f = metrics.historical_mean_return.to_f64().unwrap_or(0.0);
        let curr_wr_f = metrics.current_win_rate.to_f64().unwrap_or(0.5);
        let base_wr_f = metrics.baseline_win_rate.to_f64().unwrap_or(0.5);
        let regime_q_f = metrics.regime_quality.to_f64().unwrap_or(1.0).clamp(0.5, 1.5);

        // --- Edge Decay ---
        // How far has EMA fallen relative to historical mean?
        // edge_decay = max(0, (hist_mean - ema) / max(|hist_mean|, 1e-6))
        let edge_decay = if hist_mean_f.abs() < 1e-6 {
            0.0
        } else {
            let drop = (hist_mean_f - ema_f) / hist_mean_f.abs();
            drop.clamp(0.0, 1.0)
        };

        // --- Confidence Decay ---
        // How much has win-rate dropped from baseline?
        // confidence_decay = max(0, (base_wr - curr_wr) / max(base_wr, 0.001))
        let confidence_decay = if base_wr_f < 0.001 {
            0.0
        } else {
            let drop = (base_wr_f - curr_wr_f) / base_wr_f;
            drop.clamp(0.0, 1.0)
        };

        // --- Regime Decay ---
        // Adverse regime quality + number of transitions
        // regime_decay = (1 - regime_quality_normalised) × (1 + log(1 + transitions)/10)
        let regime_quality_norm = ((regime_q_f - 0.5) / 1.0).clamp(0.0, 1.0); // 0=adverse, 1=favourable
        let transition_factor = ((1.0 + metrics.regime_transitions as f64).ln() / 10.0).clamp(0.0, 1.0);
        let regime_decay = ((1.0 - regime_quality_norm) * (1.0 + transition_factor)).clamp(0.0, 1.0);

        // --- Composite Decay ---
        // Weighted: edge 50%, confidence 30%, regime 20%
        let composite = edge_decay * 0.50 + confidence_decay * 0.30 + regime_decay * 0.20;
        let composite = composite.clamp(0.0, 1.0);

        // --- Urgency: logistic function centred at 0.5 ---
        // urgency = 1 / (1 + e^(-12 * (composite - 0.5)))
        // → steep sigmoid: low decay → urgency ≈ 0, high decay → urgency ≈ 1
        let urgency = 1.0 / (1.0 + (-12.0 * (composite - 0.5)).exp());

        DecayOutput {
            edge_decay: to_dec(edge_decay),
            confidence_decay: to_dec(confidence_decay),
            regime_decay: to_dec(regime_decay),
            decay_score: to_dec(composite),
            urgency_score: to_dec(urgency),
        }
    }

    /// Update EMA with a new trade return.
    /// Call this after each completed trade.
    pub fn update_ema(&self, current_ema: Decimal, new_return: Decimal) -> Decimal {
        let alpha = Decimal::from_f64(self.ema_alpha).unwrap_or(Decimal::new(1, 1));
        let one_minus_alpha = Decimal::ONE - alpha;
        alpha * new_return + one_minus_alpha * current_ema
    }
}

fn to_dec(v: f64) -> Decimal {
    Decimal::from_f64(v).unwrap_or(Decimal::new(0, 0)).round_dp(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_metrics() -> DecayMetrics {
        DecayMetrics {
            ema_return: Decimal::new(80, 0),       // recent EMA at 80
            historical_mean_return: Decimal::new(100, 0), // baseline 100
            current_win_rate: Decimal::new(55, 2),  // 55%
            baseline_win_rate: Decimal::new(60, 2), // 60%
            regime_quality: Decimal::ONE,
            regime_transitions: 0,
        }
    }

    #[test]
    fn test_no_decay_when_ema_equals_mean() {
        let mut m = default_metrics();
        m.ema_return = m.historical_mean_return;
        m.current_win_rate = m.baseline_win_rate;
        let out = DecayTracker::new().compute_decay(&m);
        assert_eq!(out.edge_decay, Decimal::new(0, 0));
        assert_eq!(out.confidence_decay, Decimal::new(0, 0));
    }

    #[test]
    fn test_high_decay_triggers_high_urgency() {
        let m = DecayMetrics {
            ema_return: Decimal::new(-50, 0),      // negative EMA
            historical_mean_return: Decimal::new(100, 0),
            current_win_rate: Decimal::new(30, 2), // 30% (dropped from 60%)
            baseline_win_rate: Decimal::new(60, 2),
            regime_quality: Decimal::new(5, 1),   // 0.5 adverse
            regime_transitions: 10,
        };
        let out = DecayTracker::new().compute_decay(&m);
        let urgency_f = out.urgency_score.to_f64().unwrap_or(0.0);
        assert!(urgency_f > 0.7, "High decay should produce urgency > 0.7, got {}", urgency_f);
    }

    #[test]
    fn test_ema_update_converges() {
        let tracker = DecayTracker::new();
        let mut ema = Decimal::new(100, 0);
        // Feed constant 50 — EMA should converge toward 50
        for _ in 0..200 {
            ema = tracker.update_ema(ema, Decimal::new(50, 0));
        }
        let diff = (ema - Decimal::new(50, 0)).abs();
        assert!(diff < Decimal::new(1, 0), "EMA should converge to 50, got {}", ema);
    }
}
