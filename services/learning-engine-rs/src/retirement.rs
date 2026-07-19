use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetirementAction {
    Watchlist,
    Freeze,
    Retire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementTriggers {
    pub edge_collapse: bool,
    pub confidence_collapse: bool,
    pub prolonged_underperformance: bool,
    pub adverse_regime_shift: bool,
}

pub struct RetirementManager {
    edge_threshold: Decimal,
    confidence_threshold: Decimal,
    max_drawdown_duration: u64, // sessions or ticks
}

impl Default for RetirementManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RetirementManager {
    pub fn new() -> Self {
        Self {
            edge_threshold: Decimal::new(10, 0), // if edge score drops below 10
            confidence_threshold: Decimal::new(30, 0), // if confidence drops below 30
            max_drawdown_duration: 50,
        }
    }

    pub fn evaluate(
        &self,
        edge_score: Decimal,
        confidence: Decimal,
        drawdown_duration: u64,
        regime_drift: bool,
    ) -> Option<RetirementAction> {
        let triggers = RetirementTriggers {
            edge_collapse: edge_score < self.edge_threshold,
            confidence_collapse: confidence < self.confidence_threshold,
            prolonged_underperformance: drawdown_duration > self.max_drawdown_duration,
            adverse_regime_shift: regime_drift,
        };

        let trigger_count = [
            triggers.edge_collapse,
            triggers.confidence_collapse,
            triggers.prolonged_underperformance,
            triggers.adverse_regime_shift,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        if trigger_count >= 3 {
            Some(RetirementAction::Retire)
        } else if trigger_count == 2 {
            Some(RetirementAction::Freeze)
        } else if trigger_count == 1 {
            Some(RetirementAction::Watchlist)
        } else {
            None
        }
    }
}
