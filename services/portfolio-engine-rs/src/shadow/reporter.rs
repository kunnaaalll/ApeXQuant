use crate::shadow::statistics::ShadowStatistics;
use crate::shadow::validator::PortfolioValidationResult;
use serde::{Deserialize, Serialize};

pub struct ShadowReporter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowReport {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub report_type: ReportType,
    pub statistics: ShadowStatistics,
    pub validation: Option<PortfolioValidationResult>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    Json,
}

impl Default for ShadowReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, report_type: ReportType, stats: ShadowStatistics, validation: Option<PortfolioValidationResult>) -> ShadowReport {
        let summary = format!("Shadow Mode Report - {:?}", report_type);
        
        ShadowReport {
            generated_at: chrono::Utc::now(),
            report_type,
            statistics: stats,
            validation,
            summary,
        }
    }

    pub fn format(&self, report: &ShadowReport, format: ReportFormat) -> String {
        match format {
            ReportFormat::Json => {
                serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
            }
            ReportFormat::Markdown => {
                let val_str = if let Some(v) = &report.validation {
                    format!("Validation State: {:?}", v.state)
                } else {
                    "Validation State: N/A".to_string()
                };

                format!(
                    "# {}\nGenerated at: {}\n\n## Statistics\nAgreement: {}%\nExact Match: {}%\nAverage Drift: {}\n\n## Validation\n{}\n",
                    report.summary,
                    report.generated_at.to_rfc3339(),
                    report.statistics.agreement_percentage,
                    report.statistics.exact_match_percentage,
                    report.statistics.average_drift,
                    val_str
                )
            }
        }
    }
}
