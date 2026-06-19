use crate::shadow::drift::DriftSeverity;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ReportData {
    pub timestamp: DateTime<Utc>,
    pub agreement_percentage: Decimal,
    pub mismatch_count: u64,
    pub drift_severity: DriftSeverity,
    pub most_problematic_subsystem: String,
}

pub struct Reporter;

impl Reporter {
    pub fn generate_markdown_report(data: &ReportData) -> String {
        format!(
            "# Shadow Mode Parity Report\n\n\
            **Timestamp:** {}\n\
            **Agreement:** {}%\n\
            **Mismatches:** {}\n\
            **Drift Severity:** {:?}\n\
            **Most Problematic Subsystem:** {}\n",
            data.timestamp.to_rfc3339(),
            data.agreement_percentage,
            data.mismatch_count,
            data.drift_severity,
            data.most_problematic_subsystem
        )
    }

    pub fn generate_json_report(data: &ReportData) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(data)
    }
}
