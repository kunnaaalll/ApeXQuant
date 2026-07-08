use rust_decimal_macros::dec;

use super::failure_tracker::FailureTracker;
use super::fill_quality_guards::FillQualityGuards;
use super::latency_guards::LatencyGuards;
use super::liquidity_guards::LiquidityGuards;
use super::rejection_tracker::RejectionTracker;
use super::severity::Severity;
use super::slippage_guards::SlippageGuards;
use super::spread_guards::SpreadGuards;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    SpreadExplosion,
    SlippageSpike,
    FillDeterioration,
    LiquidityCollapse,
    LatencySpike,
    RejectionBurst,
    RepeatedTimeouts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionAnomalyReport {
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub description: String,
}

pub struct AnomalyDetectionEngine;

impl AnomalyDetectionEngine {
    pub fn detect_spread_anomalies(spread: &SpreadGuards) -> Option<ExecutionAnomalyReport> {
        let score = spread.get_score();
        if score >= 90 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::SpreadExplosion,
                severity: Severity::Catastrophic,
                description: "Spread exploded beyond 5x normal bounds".to_string(),
            })
        } else if score >= 70 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::SpreadExplosion,
                severity: Severity::Severe,
                description: "Spread multiplier exceptionally high".to_string(),
            })
        } else if score >= 50 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::SpreadExplosion,
                severity: Severity::Major,
                description: "Spread multiplier increasing significantly".to_string(),
            })
        } else {
            None
        }
    }

    pub fn detect_slippage_anomalies(slippage: &SlippageGuards) -> Option<ExecutionAnomalyReport> {
        let penalty = slippage.get_penalty_score();
        if penalty >= 90 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::SlippageSpike,
                severity: Severity::Catastrophic,
                description: "Massive slippage spike detected".to_string(),
            })
        } else if penalty >= 60 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::SlippageSpike,
                severity: Severity::Severe,
                description: "Elevated slippage detected".to_string(),
            })
        } else {
            None
        }
    }

    pub fn detect_liquidity_anomalies(
        liquidity: &LiquidityGuards,
    ) -> Option<ExecutionAnomalyReport> {
        if liquidity.available_liquidity <= dec!(0.1) || liquidity.imbalance >= dec!(0.9) {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::LiquidityCollapse,
                severity: Severity::Catastrophic,
                description: "Complete liquidity collapse on book".to_string(),
            })
        } else if liquidity.available_liquidity <= dec!(1.0) {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::LiquidityCollapse,
                severity: Severity::Severe,
                description: "Available liquidity dangerously low".to_string(),
            })
        } else {
            None
        }
    }

    pub fn detect_latency_anomalies(latency: &LatencyGuards) -> Option<ExecutionAnomalyReport> {
        let total = latency.total_latency_ms();
        if total >= 200 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::LatencySpike,
                severity: Severity::Catastrophic,
                description: "Critical latency degradation".to_string(),
            })
        } else if total >= 100 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::LatencySpike,
                severity: Severity::Major,
                description: "High execution latency detected".to_string(),
            })
        } else {
            None
        }
    }

    pub fn detect_fill_anomalies(
        fill_quality: &FillQualityGuards,
    ) -> Option<ExecutionAnomalyReport> {
        if fill_quality.total_orders >= 10 && fill_quality.fill_ratio <= dec!(0.5) {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::FillDeterioration,
                severity: Severity::Catastrophic,
                description: "Fill ratio collapsed below 50%".to_string(),
            })
        } else if fill_quality.total_orders >= 10 && fill_quality.fill_ratio <= dec!(0.8) {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::FillDeterioration,
                severity: Severity::Severe,
                description: "Significant fill deterioration".to_string(),
            })
        } else {
            None
        }
    }

    pub fn detect_rejection_anomalies(
        rejections: &RejectionTracker,
    ) -> Option<ExecutionAnomalyReport> {
        if rejections.consecutive_rejections >= 5 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::RejectionBurst,
                severity: Severity::Catastrophic,
                description: "Continuous rejection burst from broker".to_string(),
            })
        } else if rejections.consecutive_rejections >= 3 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::RejectionBurst,
                severity: Severity::Severe,
                description: "Multiple consecutive rejections".to_string(),
            })
        } else {
            None
        }
    }

    pub fn detect_failure_anomalies(failures: &FailureTracker) -> Option<ExecutionAnomalyReport> {
        if failures.timeouts >= 3 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::RepeatedTimeouts,
                severity: Severity::Catastrophic,
                description: "Repeated timeout failures".to_string(),
            })
        } else if failures.get_score() >= 80 {
            Some(ExecutionAnomalyReport {
                anomaly_type: AnomalyType::RepeatedTimeouts,
                severity: Severity::Severe,
                description: "High failure score detected".to_string(),
            })
        } else {
            None
        }
    }

    pub fn run_all_checks(
        spread: &SpreadGuards,
        slippage: &SlippageGuards,
        liquidity: &LiquidityGuards,
        latency: &LatencyGuards,
        fill_quality: &FillQualityGuards,
        rejections: &RejectionTracker,
        failures: &FailureTracker,
    ) -> Vec<ExecutionAnomalyReport> {
        let mut reports = Vec::new();

        if let Some(r) = Self::detect_spread_anomalies(spread) {
            reports.push(r);
        }
        if let Some(r) = Self::detect_slippage_anomalies(slippage) {
            reports.push(r);
        }
        if let Some(r) = Self::detect_liquidity_anomalies(liquidity) {
            reports.push(r);
        }
        if let Some(r) = Self::detect_latency_anomalies(latency) {
            reports.push(r);
        }
        if let Some(r) = Self::detect_fill_anomalies(fill_quality) {
            reports.push(r);
        }
        if let Some(r) = Self::detect_rejection_anomalies(rejections) {
            reports.push(r);
        }
        if let Some(r) = Self::detect_failure_anomalies(failures) {
            reports.push(r);
        }

        reports
    }
}
