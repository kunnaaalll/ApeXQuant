// src/analytics/performance.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PerformanceState {
    Excellent,
    Healthy,
    Normal,
    Weak,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceAssessment {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub recovery_factor: f64,
    pub ulcer_performance_index: f64,
    pub return_volatility: f64,
    pub downside_deviation: f64,
    pub stability_score: f64,
    pub state: PerformanceState,
}

impl PerformanceAssessment {
    /// Determines the portfolio's health state based on institutional stability factors.
    /// Primarily weighs the Sortino Ratio and Stability Score.
    pub fn evaluate_state(sortino: f64, stability: f64) -> PerformanceState {
        let composite_score = (sortino * 0.6) + (stability * 0.4);
        
        if composite_score < 0.0 {
            PerformanceState::Critical
        } else if composite_score < 0.5 {
            PerformanceState::Weak
        } else if composite_score < 1.0 {
            PerformanceState::Normal
        } else if composite_score < 2.0 {
            PerformanceState::Healthy
        } else {
            PerformanceState::Excellent
        }
    }

    pub fn new(
        sharpe_ratio: f64,
        sortino_ratio: f64,
        calmar_ratio: f64,
        recovery_factor: f64,
        ulcer_performance_index: f64,
        return_volatility: f64,
        downside_deviation: f64,
        stability_score: f64,
    ) -> Self {
        // Enforce math invariants (0 panics, no NaNs)
        let safe_sortino = if sortino_ratio.is_nan() { 0.0 } else { sortino_ratio };
        let safe_stability = if stability_score.is_nan() { 0.0 } else { stability_score };
        
        let state = Self::evaluate_state(safe_sortino, safe_stability);

        Self {
            sharpe_ratio: if sharpe_ratio.is_nan() { 0.0 } else { sharpe_ratio },
            sortino_ratio: safe_sortino,
            calmar_ratio: if calmar_ratio.is_nan() { 0.0 } else { calmar_ratio },
            recovery_factor: if recovery_factor.is_nan() { 0.0 } else { recovery_factor },
            ulcer_performance_index: if ulcer_performance_index.is_nan() { 0.0 } else { ulcer_performance_index },
            return_volatility: if return_volatility.is_nan() { 0.0 } else { return_volatility },
            downside_deviation: if downside_deviation.is_nan() { 0.0 } else { downside_deviation },
            stability_score: safe_stability,
            state,
        }
    }
}
