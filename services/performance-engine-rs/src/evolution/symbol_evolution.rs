use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEvolutionWindow {
    pub symbol: String,
    pub period_label: String,
    pub trade_count: u32,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolTrend {
    Strengthening,
    Stable,
    Weakening,
    Exhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEvolutionReport {
    pub symbol: String,
    pub trend: SymbolTrend,
    pub expectancy_drift: Decimal,
    pub drawdown_drift: Decimal,
    pub periods_analyzed: usize,
    pub explanation: String,
}

pub struct SymbolEvolutionEngine;

impl SymbolEvolutionEngine {
    /// Evaluate symbol performance evolution over ordered windows (oldest → newest).
    pub fn evaluate(windows: &[SymbolEvolutionWindow]) -> Option<SymbolEvolutionReport> {
        if windows.len() < 2 {
            return None;
        }

        let symbol = windows[0].symbol.clone();
        let first = &windows[0];
        let last = windows.last().unwrap();

        let expectancy_drift = last.expectancy - first.expectancy;
        // Drawdown is a positive magnitude; increase = worse
        let drawdown_drift = last.max_drawdown - first.max_drawdown;

        let trend = if last.trade_count == 0 {
            SymbolTrend::Exhausted
        } else if expectancy_drift >= dec!(0.05) && drawdown_drift <= dec!(0) {
            SymbolTrend::Strengthening
        } else if expectancy_drift <= dec!(-0.05) || drawdown_drift >= dec!(0.05) {
            SymbolTrend::Weakening
        } else {
            SymbolTrend::Stable
        };

        let explanation = format!(
            "Symbol {} expectancy drifted {} over {} periods. Drawdown drift: {}. Trend: {:?}.",
            symbol, expectancy_drift, windows.len(), drawdown_drift, trend
        );

        Some(SymbolEvolutionReport {
            symbol,
            trend,
            expectancy_drift,
            drawdown_drift,
            periods_analyzed: windows.len(),
            explanation,
        })
    }
}
