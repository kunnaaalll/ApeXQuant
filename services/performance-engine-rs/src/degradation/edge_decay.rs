use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Edge decay tracks sustained reduction in risk-adjusted returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeDecayState {
    Healthy,
    Weakening,
    Warning,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDecaySnapshot {
    pub period_label: String,
    pub edge_score: Decimal, // 0.0 – 1.0 normalized edge strength
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDecayReport {
    pub state: EdgeDecayState,
    pub total_decay: Decimal,
    pub decay_rate_per_period: Decimal,
    pub periods_analyzed: usize,
    pub explanation: String,
}

pub struct EdgeDecayEngine;

impl EdgeDecayEngine {
    /// Evaluate the rate and magnitude of edge decay from ordered snapshots.
    pub fn evaluate(snapshots: &[EdgeDecaySnapshot]) -> Option<EdgeDecayReport> {
        if snapshots.len() < 2 {
            return None;
        }

        let first = &snapshots[0];
        let last = snapshots.last().unwrap();
        let n = snapshots.len();

        let total_decay = first.edge_score - last.edge_score;
        let decay_rate_per_period = if n > 1 {
            total_decay / Decimal::from(n as u64 - 1)
        } else {
            Decimal::ZERO
        };

        let state = if last.edge_score < dec!(0.20) || total_decay > dec!(0.60) {
            EdgeDecayState::Collapse
        } else if total_decay > dec!(0.40) || decay_rate_per_period > dec!(0.08) {
            EdgeDecayState::Critical
        } else if total_decay > dec!(0.25) || decay_rate_per_period > dec!(0.05) {
            EdgeDecayState::Warning
        } else if total_decay > dec!(0.10) {
            EdgeDecayState::Weakening
        } else {
            EdgeDecayState::Healthy
        };

        let explanation = format!(
            "Edge decay: total={}, rate/period={}, state={:?} over {} periods.",
            total_decay, decay_rate_per_period, state, n
        );

        Some(EdgeDecayReport {
            state,
            total_decay,
            decay_rate_per_period,
            periods_analyzed: n,
            explanation,
        })
    }
}
