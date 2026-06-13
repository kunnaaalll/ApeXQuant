//! Configuration management for the Signal Engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Signal Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Quality thresholds
    pub min_confluence_score: u8,
    pub min_signal_quality: SignalQuality,
    pub min_risk_reward: f64,

    /// Timeframes to analyze
    pub timeframes: Vec<String>,
    /// Primary execution timeframe
    pub execution_timeframe: String,

    /// Regime detection parameters
    pub volatility_lookback: usize,
    pub volatility_percentile_threshold: f64,
    pub trend_lookback: usize,

    /// Pattern parameters
    pub swing_pivot_bars: usize,
    pub ob_max_age_bars: usize,
    pub fvg_tolerance_percent: f64,
    pub min_displacement_atr_multiple: f64,

    /// Confluence weights
    pub confluence_weights: ConfluenceWeights,

    /// Feature flags
    pub filters: FilterConfig,

    /// Shadow mode
    pub shadow_mode: ShadowModeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_confluence_score: 70,
            min_signal_quality: SignalQuality::A,
            min_risk_reward: 2.0,
            timeframes: vec![
                "H4".to_string(),
                "H1".to_string(),
                "M30".to_string(),
                "M15".to_string(),
            ],
            execution_timeframe: "M15".to_string(),
            volatility_lookback: 50,
            volatility_percentile_threshold: 0.75,
            trend_lookback: 20,
            swing_pivot_bars: 3,
            ob_max_age_bars: 30,
            fvg_tolerance_percent: 0.1,
            min_displacement_atr_multiple: 1.5,
            confluence_weights: ConfluenceWeights::default(),
            filters: FilterConfig::default(),
            shadow_mode: ShadowModeConfig::default(),
        }
    }
}

/// Signal quality grades
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalQuality {
    /// A+ grade: Excellent signal
    APlus,
    /// A grade: Good signal
    A,
    /// B grade: Marginal signal (not emitted)
    B,
    /// Rejected: Below quality threshold
    Reject,
}

impl SignalQuality {
    /// Check if this quality meets the minimum threshold
    pub fn meets(&self, minimum: SignalQuality) -> bool {
        let value = match self {
            SignalQuality::APlus => 3,
            SignalQuality::A => 2,
            SignalQuality::B => 1,
            SignalQuality::Reject => 0,
        };
        let min_value = match minimum {
            SignalQuality::APlus => 3,
            SignalQuality::A => 2,
            SignalQuality::B => 1,
            SignalQuality::Reject => 0,
        };
        value >= min_value
    }
}

/// Confluence factor weights (should sum to 1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceWeights {
    pub htf_alignment: f64,
    pub regime: f64,
    pub momentum: f64,
    pub liquidity: f64,
    pub order_block: f64,
    pub fvg: f64,
    pub displacement: f64,
    pub session: f64,
    pub structure_quality: f64,
    pub trend_strength: f64,
}

impl Default for ConfluenceWeights {
    fn default() -> Self {
        Self {
            htf_alignment: 0.20,
            regime: 0.15,
            momentum: 0.12,
            liquidity: 0.12,
            order_block: 0.10,
            fvg: 0.08,
            displacement: 0.08,
            session: 0.07,
            structure_quality: 0.05,
            trend_strength: 0.03,
        }
    }
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub session_filter: bool,
    pub regime_filter: bool,
    pub duplicate_suppression_seconds: u64,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            session_filter: true,
            regime_filter: true,
            duplicate_suppression_seconds: 300,
        }
    }
}

/// Shadow mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowModeConfig {
    pub enabled: bool,
    pub emit_to_pubsub: bool,
    pub store_comparisons: bool,
    pub comparison_sample_rate: f64,
}

impl Default for ShadowModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            emit_to_pubsub: true,
            store_comparisons: true,
            comparison_sample_rate: 1.0,
        }
    }
}

impl Config {
    /// Load configuration from file and environment
    pub fn load() -> crate::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("signal-engine").required(false))
            .add_source(config::Environment::with_prefix("APEX_SIGNAL"))
            .build()?;

        Ok(settings.try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_quality_comparison() {
        assert!(SignalQuality::APlus.meets(SignalQuality::A));
        assert!(SignalQuality::A.meets(SignalQuality::A));
        assert!(!SignalQuality::B.meets(SignalQuality::A));
        assert!(!SignalQuality::Reject.meets(SignalQuality::A));
    }

    #[test]
    fn confluence_weights_sum_to_one() {
        let weights = ConfluenceWeights::default();
        let sum = weights.htf_alignment
            + weights.regime
            + weights.momentum
            + weights.liquidity
            + weights.order_block
            + weights.fvg
            + weights.displacement
            + weights.session
            + weights.structure_quality
            + weights.trend_strength;

        assert!((sum - 1.0).abs() < 0.001);
    }
}
