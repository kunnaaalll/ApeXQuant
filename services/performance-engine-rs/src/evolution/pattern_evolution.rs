use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEvolutionWindow {
    pub pattern_id: String,
    pub period_label: String,
    pub trade_count: u32,
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub avg_rr: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternTrend {
    Maturing,
    Stable,
    Fading,
    Obsolete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEvolutionReport {
    pub pattern_id: String,
    pub trend: PatternTrend,
    pub expectancy_drift: Decimal,
    pub win_rate_drift: Decimal,
    pub periods_analyzed: usize,
    pub explanation: String,
}

pub struct PatternEvolutionEngine;

impl PatternEvolutionEngine {
    /// Evaluate how a pattern's edge has evolved over ordered windows (oldest → newest).
    pub fn evaluate(windows: &[PatternEvolutionWindow]) -> Option<PatternEvolutionReport> {
        if windows.len() < 2 {
            return None;
        }

        let pattern_id = windows[0].pattern_id.clone();
        let first = &windows[0];
        let last = windows.last()?;

        let expectancy_drift = last.expectancy - first.expectancy;
        let win_rate_drift = last.win_rate - first.win_rate;

        let trend = if last.trade_count == 0 {
            PatternTrend::Obsolete
        } else if expectancy_drift >= dec!(0.05) {
            PatternTrend::Maturing
        } else if expectancy_drift <= dec!(-0.05) || win_rate_drift <= dec!(-0.05) {
            PatternTrend::Fading
        } else {
            PatternTrend::Stable
        };

        let explanation =
            format!(
            "Pattern {} expectancy drifted {} over {} periods. Win-rate drift: {}. Trend: {:?}.",
            pattern_id, expectancy_drift, windows.len(), win_rate_drift, trend
        );

        Some(PatternEvolutionReport {
            pattern_id,
            trend,
            expectancy_drift,
            win_rate_drift,
            periods_analyzed: windows.len(),
            explanation,
        })
    }
}
