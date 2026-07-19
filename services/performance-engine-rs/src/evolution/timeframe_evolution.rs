use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeEvolutionWindow {
    pub timeframe: String,
    pub period_label: String,
    pub trade_count: u32,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub stability: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeframeTrend {
    Strengthening,
    Stable,
    Weakening,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeEvolutionReport {
    pub timeframe: String,
    pub trend: TimeframeTrend,
    pub expectancy_drift: Decimal,
    pub stability_drift: Decimal,
    pub periods_analyzed: usize,
    pub explanation: String,
}

pub struct TimeframeEvolutionEngine;

impl TimeframeEvolutionEngine {
    /// Evaluate timeframe performance evolution over ordered windows (oldest → newest).
    pub fn evaluate(windows: &[TimeframeEvolutionWindow]) -> Option<TimeframeEvolutionReport> {
        if windows.len() < 2 {
            return None;
        }

        let timeframe = windows[0].timeframe.clone();
        let first = &windows[0];
        let last = windows.last()?;

        let expectancy_drift = last.expectancy - first.expectancy;
        let stability_drift = last.stability - first.stability;

        let trend = if last.trade_count == 0 {
            TimeframeTrend::Abandoned
        } else if expectancy_drift >= dec!(0.05) && stability_drift >= dec!(0) {
            TimeframeTrend::Strengthening
        } else if expectancy_drift <= dec!(-0.05) || stability_drift <= dec!(-0.05) {
            TimeframeTrend::Weakening
        } else {
            TimeframeTrend::Stable
        };

        let explanation =
            format!(
            "Timeframe {} expectancy drifted {} over {} periods. Stability drift: {}. Trend: {:?}.",
            timeframe, expectancy_drift, windows.len(), stability_drift, trend
        );

        Some(TimeframeEvolutionReport {
            timeframe,
            trend,
            expectancy_drift,
            stability_drift,
            periods_analyzed: windows.len(),
            explanation,
        })
    }
}
