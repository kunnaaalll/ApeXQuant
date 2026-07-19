use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftStatus {
    Stable,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMetrics {
    pub current_deviation: Decimal,
    pub warning_threshold: Decimal,
    pub critical_threshold: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    pub strategy_drift: DriftStatus,
    pub market_drift: DriftStatus,
    pub execution_drift: DriftStatus,
    pub risk_drift: DriftStatus,
}

pub struct DriftMonitor;

impl DriftMonitor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DriftMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftMonitor {
    fn evaluate_metric(metric: &DriftMetrics) -> DriftStatus {
        if metric.current_deviation >= metric.critical_threshold {
            DriftStatus::Critical
        } else if metric.current_deviation >= metric.warning_threshold {
            DriftStatus::Warning
        } else {
            DriftStatus::Stable
        }
    }

    pub fn analyze(
        &self,
        strategy: DriftMetrics,
        market: DriftMetrics,
        execution: DriftMetrics,
        risk: DriftMetrics,
    ) -> DriftAnalysis {
        DriftAnalysis {
            strategy_drift: Self::evaluate_metric(&strategy),
            market_drift: Self::evaluate_metric(&market),
            execution_drift: Self::evaluate_metric(&execution),
            risk_drift: Self::evaluate_metric(&risk),
        }
    }
}
