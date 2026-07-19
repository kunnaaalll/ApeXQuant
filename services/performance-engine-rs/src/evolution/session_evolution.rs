use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvolutionWindow {
    pub session: String,
    pub period_label: String,
    pub trade_count: u32,
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub avg_rr: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionTrend {
    Improving,
    Stable,
    Deteriorating,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvolutionReport {
    pub session: String,
    pub trend: SessionTrend,
    pub expectancy_drift: Decimal,
    pub win_rate_drift: Decimal,
    pub periods_analyzed: usize,
    pub explanation: String,
}

pub struct SessionEvolutionEngine;

impl SessionEvolutionEngine {
    /// Evaluate session performance evolution over ordered windows (oldest → newest).
    pub fn evaluate(windows: &[SessionEvolutionWindow]) -> Option<SessionEvolutionReport> {
        if windows.len() < 2 {
            return None;
        }

        let session = windows[0].session.clone();
        let first = &windows[0];
        let last = windows.last()?;

        let expectancy_drift = last.expectancy - first.expectancy;
        let win_rate_drift = last.win_rate - first.win_rate;

        let trend = if last.trade_count == 0 {
            SessionTrend::Abandoned
        } else if expectancy_drift >= dec!(0.05) && win_rate_drift >= dec!(0) {
            SessionTrend::Improving
        } else if expectancy_drift <= dec!(-0.05) || win_rate_drift <= dec!(-0.05) {
            SessionTrend::Deteriorating
        } else {
            SessionTrend::Stable
        };

        let explanation =
            format!(
            "Session {} expectancy drifted {} over {} periods. Win-rate drift: {}. Trend: {:?}.",
            session, expectancy_drift, windows.len(), win_rate_drift, trend
        );

        Some(SessionEvolutionReport {
            session,
            trend,
            expectancy_drift,
            win_rate_drift,
            periods_analyzed: windows.len(),
            explanation,
        })
    }
}
