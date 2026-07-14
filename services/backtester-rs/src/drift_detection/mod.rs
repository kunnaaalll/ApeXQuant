//! Drift Detection Module
//!
//! Detects operational and performance drift over time.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum DriftSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DriftDirection {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct DriftReport {
    pub severity: DriftSeverity,
    pub direction: DriftDirection,
    pub confidence: Decimal,
    pub drift_value: Decimal,
    pub metric_name: String,
}

pub trait DriftDetector {
    type InputSeries;
    fn detect_drift(
        &self,
        historical: &Self::InputSeries,
        recent: &Self::InputSeries,
    ) -> DriftReport;
}

pub struct MarketBehaviourDriftDetector {
    pub threshold: Decimal,
}

impl MarketBehaviourDriftDetector {
    pub fn new(threshold: Decimal) -> Self {
        Self { threshold }
    }
}

pub struct StrategyPerformanceDriftDetector {
    pub threshold: Decimal,
}

impl StrategyPerformanceDriftDetector {
    pub fn new(threshold: Decimal) -> Self {
        Self { threshold }
    }
}

pub struct ExecutionQualityDriftDetector {
    pub threshold: Decimal,
}

impl ExecutionQualityDriftDetector {
    pub fn new(threshold: Decimal) -> Self {
        Self { threshold }
    }
}

pub struct PortfolioBehaviourDriftDetector {
    pub threshold: Decimal,
}

impl PortfolioBehaviourDriftDetector {
    pub fn new(threshold: Decimal) -> Self {
        Self { threshold }
    }
}
