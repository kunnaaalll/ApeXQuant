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
    ) -> Option<AnomalyReport> {
        let three = Decimal::new(3, 0);
        let five = Decimal::new(5, 0);
        
        let ratio = if historical_avg_slippage.is_zero() {
            Decimal::ZERO
        } else {
            current_slippage / historical_avg_slippage
        };

        let severity = if ratio > five {
            AnomalySeverity::Critical
        } else if ratio > three {
            AnomalySeverity::High
        } else {
            return None;
        };

        Some(AnomalyReport {
            anomaly_type: "abnormal_slippage".to_string(),
            severity,
            confidence: Decimal::new(95, 0), // Highly confident if ratio is large
            details: format!("Current slippage {} is {}x historical average", current_slippage, ratio),
        })
    }
}
