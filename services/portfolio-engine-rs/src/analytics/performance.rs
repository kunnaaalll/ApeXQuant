// src/analytics/performance.rs
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

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
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub recovery_factor: Decimal,
    pub ulcer_performance_index: Decimal,
    pub return_volatility: Decimal,
    pub downside_deviation: Decimal,
    pub stability_score: Decimal,
    pub state: PerformanceState,
}

impl PerformanceAssessment {
    /// Determines the portfolio's health state based on institutional stability factors.
    /// Primarily weighs the Sortino Ratio and Stability Score.
    pub fn evaluate_state(sortino: Decimal, stability: Decimal) -> PerformanceState {
        let weight_sortino = Decimal::from_f32(0.6).unwrap_or(Decimal::ZERO);
        let weight_stability = Decimal::from_f32(0.4).unwrap_or(Decimal::ZERO);
        
        let composite_score = (sortino * weight_sortino) + (stability * weight_stability);
        
        if composite_score < Decimal::ZERO {
            PerformanceState::Critical
        } else if composite_score < Decimal::from_f32(0.5).unwrap_or(Decimal::ZERO) {
            PerformanceState::Weak
        } else if composite_score < Decimal::ONE {
            PerformanceState::Normal
        } else if composite_score < Decimal::from_f32(2.0).unwrap_or(Decimal::ZERO) {
            PerformanceState::Healthy
        } else {
            PerformanceState::Excellent
        }
    }

    pub fn new(
        sharpe_ratio: Decimal,
        sortino_ratio: Decimal,
        calmar_ratio: Decimal,
        recovery_factor: Decimal,
        ulcer_performance_index: Decimal,
        return_volatility: Decimal,
        downside_deviation: Decimal,
        stability_score: Decimal,
    ) -> Self {
        let state = Self::evaluate_state(sortino_ratio, stability_score);

        Self {
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            recovery_factor,
            ulcer_performance_index,
            return_volatility,
            downside_deviation,
            stability_score,
            state,
        }
    }
}
