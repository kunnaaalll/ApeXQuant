use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Rolling-window performance snapshot for a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDegradationWindow {
    pub period_label: String,
    pub trade_count: u32,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub confidence: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationState {
    Healthy,
    Weakening,
    Warning,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDegradationReport {
    pub state: DegradationState,
    pub expectancy_decline: Decimal,
    pub profit_factor_decline: Decimal,
    pub confidence_decline: Decimal,
    pub explanation: String,
}

pub struct StrategyDegradationEngine;

impl StrategyDegradationEngine {
    /// Collapse is immediate; recovery requires multiple healthy windows.
    pub fn evaluate(windows: &[StrategyDegradationWindow]) -> Option<StrategyDegradationReport> {
        if windows.len() < 2 {
            return None;
        }

        let first = &windows[0];
        let last = windows.last().unwrap();

        let expectancy_decline = first.expectancy - last.expectancy;
        let profit_factor_decline = first.profit_factor - last.profit_factor;
        let confidence_decline = first.confidence - last.confidence;

        // Collapse is triggered immediately on severe breach
        let state = if last.expectancy < dec!(-0.05)
            || last.profit_factor < dec!(0.8)
            || confidence_decline > dec!(0.40)
        {
            DegradationState::Collapse
        } else if expectancy_decline > dec!(0.15) || profit_factor_decline > dec!(0.4) {
            DegradationState::Critical
        } else if expectancy_decline > dec!(0.08) || profit_factor_decline > dec!(0.2) {
            DegradationState::Warning
        } else if expectancy_decline > dec!(0.03) || profit_factor_decline > dec!(0.1) {
            DegradationState::Weakening
        } else {
            DegradationState::Healthy
        };

        let explanation = format!(
            "Strategy degradation state: {:?}. Expectancy decline: {}, PF decline: {}, confidence decline: {}.",
            state, expectancy_decline, profit_factor_decline, confidence_decline
        );

        Some(StrategyDegradationReport {
            state,
            expectancy_decline,
            profit_factor_decline,
            confidence_decline,
            explanation,
        })
    }
}
