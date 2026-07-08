use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub anomaly_type: String, // e.g. "unexpected_losses", "abnormal_slippage", "abnormal_drawdown", "unusual_correlations"
    pub severity: AnomalySeverity,
    pub confidence: Decimal, // 0 to 100
    pub details: String,
}

pub struct AnomalyDetector;

impl AnomalyDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_slippage_anomaly(
        &self,
        current_slippage: Decimal,
        historical_avg_slippage: Decimal,
        historical_std_dev: Decimal,
    ) -> Option<AnomalyReport> {
        let z_score = if historical_std_dev.is_zero() {
            // Fallback if no variance: if current is more than 3x avg, alert
            if current_slippage > historical_avg_slippage * Decimal::new(3, 0) {
                Decimal::new(3, 0)
            } else {
                return None;
            }
        } else {
            (current_slippage - historical_avg_slippage) / historical_std_dev
        };

        let severity = if z_score > Decimal::new(5, 0) {
            AnomalySeverity::Critical
        } else if z_score > Decimal::new(3, 0) {
            AnomalySeverity::High
        } else if z_score > Decimal::new(2, 0) {
            AnomalySeverity::Medium
        } else {
            return None;
        };

        Some(AnomalyReport {
            anomaly_type: "abnormal_slippage".to_string(),
            severity,
            confidence: Decimal::new(95, 0), // Highly confident if ratio is large
            details: format!("Current slippage {} has z-score of {} compared to historical std dev", current_slippage, z_score),
        })
    }
}
