use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Tracks how a specific market regime's performance has evolved over rolling windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeEvolutionWindow {
    pub regime_id: String,
    pub period_label: String,
    pub trade_count: u32,
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegimeTrend {
    Strengthening,
    Stable,
    Weakening,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeEvolutionReport {
    pub regime_id: String,
    pub trend: RegimeTrend,
    pub expectancy_drift: Decimal,
    pub win_rate_drift: Decimal,
    pub periods_analyzed: usize,
    pub explanation: String,
}

pub struct RegimeEvolutionEngine;

impl RegimeEvolutionEngine {
    /// Evaluate evolution over ordered windows (oldest → newest).
    pub fn evaluate(windows: &[RegimeEvolutionWindow]) -> Option<RegimeEvolutionReport> {
        if windows.len() < 2 {
            return None;
        }

        let regime_id = windows[0].regime_id.clone();
        let first = &windows[0];
        let last = windows.last().unwrap();

        let expectancy_drift = last.expectancy - first.expectancy;
        let win_rate_drift = last.win_rate - first.win_rate;

        let trend = if last.trade_count == 0 {
            RegimeTrend::Abandoned
        } else if expectancy_drift >= dec!(0.05) && win_rate_drift >= dec!(0) {
            RegimeTrend::Strengthening
        } else if expectancy_drift <= dec!(-0.05) || win_rate_drift <= dec!(-0.05) {
            RegimeTrend::Weakening
        } else {
            RegimeTrend::Stable
        };

        let explanation = format!(
            "Regime {} evolved from expectancy {} to {} over {} periods. Trend: {:?}.",
            regime_id, first.expectancy, last.expectancy, windows.len(), trend
        );

        Some(RegimeEvolutionReport {
            regime_id,
            trend,
            expectancy_drift,
            win_rate_drift,
            periods_analyzed: windows.len(),
            explanation,
        })
    }
}
