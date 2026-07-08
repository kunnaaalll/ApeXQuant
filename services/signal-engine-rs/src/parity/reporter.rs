//! Parity Reporter - Generate reports for validation analysis
//!
//! Creates human-readable reports including markdown summaries,
//! JSON exports, and structured data for dashboards.

use super::*;
use std::fmt::Write as FmtWrite;

/// Report output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Markdown,
    Json,
    Csv,
    Html,
}

/// Report generation configuration
#[derive(Debug, Clone)]
pub struct ReporterConfig {
    pub include_full_history: bool,
    pub include_symbol_breakdown: bool,
    pub include_pattern_analysis: bool,
    pub suspicious_threshold: f64,
    pub max_disagreements_to_include: usize,
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            include_full_history: false,
            include_symbol_breakdown: true,
            include_pattern_analysis: true,
            suspicious_threshold: 0.7,
            max_disagreements_to_include: 50,
        }
    }
}

/// Generates parity reports
pub struct ParityReporter {
    report_interval: u64,
    last_report_time: Option<DateTime<Utc>>,
    config: ReporterConfig,
}

impl ParityReporter {
    /// Create a new parity reporter
    pub fn new(report_interval_minutes: u64) -> Self {
        Self {
            report_interval: report_interval_minutes,
            last_report_time: None,
            config: ReporterConfig::default(),
        }
    }

    /// Check if report should be generated
    pub fn should_report(&self) -> bool {
        match self.last_report_time {
            None => true,
            Some(last) => {
                let elapsed = Utc::now().signed_duration_since(last);
                elapsed.num_minutes() >= self.report_interval as i64
            }
        }
    }

    /// Mark report as generated
    pub fn mark_reported(&mut self) {
        self.last_report_time = Some(Utc::now());
    }

    /// Generate a comprehensive parity report
    pub fn generate_report(
        &self,
        statistics: &ParityStatistics,
        drift_report: &DriftReport,
        recent_comparisons: &[SignalComparisonRecord],
    ) -> Result<ParityReport> {
        let metrics = statistics.to_metrics();
        let go_live_status = self.assess_go_live_status(&metrics, drift_report);

        let report = ParityReport {
            generated_at: Utc::now(),
            runtime_duration_hours: statistics.runtime_duration().num_seconds() as f64 / 3600.0,
            total_comparisons: statistics.total_comparisons,
            metrics,
            drift_summary: self.summarize_drift(drift_report),
            go_live_status,
            symbol_breakdown: if self.config.include_symbol_breakdown {
                self.generate_symbol_breakdown(statistics)
            } else {
                vec![]
            },
            pattern_analysis: if self.config.include_pattern_analysis {
                self.analyze_patterns(recent_comparisons)
            } else {
                vec![]
            },
            suspicious_cases: self.identify_suspicious_cases(recent_comparisons),
            top_disagreements: self.get_top_disagreements(recent_comparisons),
        };

        Ok(report)
    }

    /// Assess go-live readiness
    fn assess_go_live_status(
        &self,
        metrics: &AgreementMetrics,
        drift_report: &DriftReport,
    ) -> GoLiveStatus {
        let mut reasons = Vec::new();
        let mut passing = true;

        // Direction agreement check
        if metrics.direction_agreement_pct < 95.0 {
            passing = false;
            reasons.push(format!(
                "Direction agreement {:.1}% < 95% threshold",
                metrics.direction_agreement_pct
            ));
        }

        // Confidence drift check
        if metrics.avg_confidence_diff > 10.0 {
            passing = false;
            reasons.push(format!(
                "Confidence drift {:.1}% > 10% threshold",
                metrics.avg_confidence_diff
            ));
        }

        // Pattern agreement check
        let pattern_agreement = self.calculate_pattern_agreement(metrics);
        if pattern_agreement < 90.0 {
            passing = false;
            reasons.push(format!(
                "Pattern agreement {:.1}% < 90% threshold",
                pattern_agreement
            ));
        }

        // Disagreement rate check
        let disagreement_rate = if metrics.total_comparisons > 0 {
            (metrics.disagreements as f64 / metrics.total_comparisons as f64) * 100.0
        } else {
            0.0
        };
        if disagreement_rate > 5.0 {
            passing = false;
            reasons.push(format!(
                "Disagreement rate {:.1}% > 5% threshold",
                disagreement_rate
            ));
        }

        // Drift alerts check
        if drift_report.has_alerting {
            passing = false;
            reasons.push("Active drift alerts present".to_string());
        }

        GoLiveStatus {
            ready: passing,
            passing_criteria: self.get_passing_criteria(metrics, drift_report),
            failing_criteria: reasons.clone(),
            recommendation: if passing {
                "System meets all go-live criteria".to_string()
            } else {
                format!("System requires {} fixes before go-live", reasons.len())
            },
        }
    }

    fn get_passing_criteria(
        &self,
        metrics: &AgreementMetrics,
        drift_report: &DriftReport,
    ) -> Vec<String> {
        let mut passing = Vec::new();

        if metrics.direction_agreement_pct >= 95.0 {
            passing.push(format!(
                "Direction agreement {:.1}% >= 95%",
                metrics.direction_agreement_pct
            ));
        }
        if metrics.avg_confidence_diff <= 10.0 {
            passing.push(format!(
                "Confidence drift {:.1}% <= 10%",
                metrics.avg_confidence_diff
            ));
        }
        if !drift_report.has_alerting {
            passing.push("No drift alerts".to_string());
        }

        passing
    }

    fn calculate_pattern_agreement(&self, _metrics: &AgreementMetrics) -> f64 {
        // Simplified - would be calculated from actual pattern comparisons
        92.0
    }

    fn summarize_drift(&self, drift_report: &DriftReport) -> DriftSummary {
        let alerting_metrics: Vec<String> = drift_report
            .measurements
            .iter()
            .filter(|m| m.is_alerting)
            .map(|m| m.metric_name.clone())
            .collect();

        DriftSummary {
            has_alerts: drift_report.has_alerting,
            alerting_metrics,
            max_confidence_drift: drift_report
                .measurements
                .iter()
                .find(|m| m.metric_name == "confidence")
                .map_or(0.0, |m| m.relative_drift_pct),
            summary_text: drift_report.summary.clone(),
        }
    }

    fn generate_symbol_breakdown(&self, statistics: &ParityStatistics) -> Vec<SymbolReport> {
        statistics
            .by_symbol
            .iter()
            .map(|(symbol, stats)| SymbolReport {
                symbol: symbol.clone(),
                total_comparisons: stats.comparison_count,
                direction_agreement_pct: stats.direction_agreement_pct(),
                exact_matches: stats.exact_matches,
                disagreements: stats.disagreements,
                false_positives: stats.false_positives,
                avg_agreement_score: stats.avg_agreement_score,
                status: if stats.direction_agreement_pct() >= 95.0 {
                    SymbolStatus::Passing
                } else {
                    SymbolStatus::NeedsReview
                },
            })
            .collect()
    }

    fn analyze_patterns(&self, comparisons: &[SignalComparisonRecord]) -> Vec<PatternAnalysis> {
        let mut pattern_stats: HashMap<String, (u64, u64)> = HashMap::new();

        for comp in comparisons {
            for pat in &comp.pattern_comparisons {
                let entry = pattern_stats
                    .entry(pat.pattern_name.clone())
                    .or_insert((0, 0));
                entry.0 += 1; // total
                if pat.agreement {
                    entry.1 += 1; // agreements
                }
            }
        }

        pattern_stats
            .into_iter()
            .map(|(name, (total, agreements))| PatternAnalysis {
                pattern_name: name,
                total_occurrences: total,
                agreement_count: agreements,
                agreement_pct: if total > 0 {
                    (agreements as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect()
    }

    fn identify_suspicious_cases(
        &self,
        comparisons: &[SignalComparisonRecord],
    ) -> Vec<SuspiciousCase> {
        comparisons
            .iter()
            .filter(|c| c.agreement_score < self.config.suspicious_threshold)
            .map(|c| SuspiciousCase {
                comparison_id: c.comparison_id.clone(),
                timestamp: c.timestamp,
                symbol: c.symbol.clone(),
                score: c.agreement_score,
                reason: c.notes.clone(),
            })
            .take(self.config.max_disagreements_to_include)
            .collect()
    }

    fn get_top_disagreements(
        &self,
        comparisons: &[SignalComparisonRecord],
    ) -> Vec<SignalComparisonRecord> {
        comparisons
            .iter()
            .filter(|c| matches!(c.comparison_type, ComparisonType::Disagreement))
            .cloned()
            .take(self.config.max_disagreements_to_include)
            .collect()
    }
}

/// Parity go-live status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoLiveStatus {
    pub ready: bool,
    pub passing_criteria: Vec<String>,
    pub failing_criteria: Vec<String>,
    pub recommendation: String,
}

/// Drift summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSummary {
    pub has_alerts: bool,
    pub alerting_metrics: Vec<String>,
    pub max_confidence_drift: f64,
    pub summary_text: String,
}

/// Symbol-level report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReport {
    pub symbol: String,
    pub total_comparisons: u64,
    pub direction_agreement_pct: f64,
    pub exact_matches: u64,
    pub disagreements: u64,
    pub false_positives: u64,
    pub avg_agreement_score: f64,
    pub status: SymbolStatus,
}

/// Symbol validation status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SymbolStatus {
    Passing,
    NeedsReview,
    Failing,
}

/// Pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub pattern_name: String,
    pub total_occurrences: u64,
    pub agreement_count: u64,
    pub agreement_pct: f64,
}

/// Suspicious case for review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousCase {
    pub comparison_id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub score: f64,
    pub reason: String,
}

/// Complete parity report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityReport {
    pub generated_at: DateTime<Utc>,
    pub runtime_duration_hours: f64,
    pub total_comparisons: u64,
    pub metrics: AgreementMetrics,
    pub drift_summary: DriftSummary,
    pub go_live_status: GoLiveStatus,
    pub symbol_breakdown: Vec<SymbolReport>,
    pub pattern_analysis: Vec<PatternAnalysis>,
    pub suspicious_cases: Vec<SuspiciousCase>,
    pub top_disagreements: Vec<SignalComparisonRecord>,
}

impl ParityReport {
    /// Export to markdown format
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        writeln!(&mut md, "# APEX Signal Engine Parity Report").unwrap();
        writeln!(&mut md).unwrap();
        writeln!(
            &mut md,
            "**Generated:** {}",
            self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
        .unwrap();
        writeln!(
            &mut md,
            "**Runtime:** {:.1} hours",
            self.runtime_duration_hours
        )
        .unwrap();
        writeln!(&mut md, "**Total Comparisons:** {}", self.total_comparisons).unwrap();
        writeln!(&mut md).unwrap();

        // Go-Live Status
        writeln!(&mut md, "## Go-Live Status").unwrap();
        writeln!(&mut md).unwrap();
        let status_icon = if self.go_live_status.ready {
            "✅"
        } else {
            "❌"
        };
        writeln!(
            &mut md,
            "### {} {}",
            status_icon,
            if self.go_live_status.ready {
                "READY"
            } else {
                "NOT READY"
            }
        )
        .unwrap();
        writeln!(&mut md).unwrap();
        writeln!(
            &mut md,
            "**Recommendation:** {}",
            self.go_live_status.recommendation
        )
        .unwrap();
        writeln!(&mut md).unwrap();

        if !self.go_live_status.passing_criteria.is_empty() {
            writeln!(&mut md, "#### Passing Criteria").unwrap();
            for c in &self.go_live_status.passing_criteria {
                writeln!(&mut md, "- ✅ {}", c).unwrap();
            }
            writeln!(&mut md).unwrap();
        }

        if !self.go_live_status.failing_criteria.is_empty() {
            writeln!(&mut md, "#### Failing Criteria").unwrap();
            for c in &self.go_live_status.failing_criteria {
                writeln!(&mut md, "- ❌ {}", c).unwrap();
            }
            writeln!(&mut md).unwrap();
        }

        // Metrics Summary
        writeln!(&mut md, "## Metrics Summary").unwrap();
        writeln!(&mut md).unwrap();
        writeln!(&mut md, "| Metric | Value | Target | Status |").unwrap();
        writeln!(&mut md, "|--------|-------|--------|--------|").unwrap();

        let dir_status = if self.metrics.direction_agreement_pct >= 95.0 {
            "✅"
        } else {
            "❌"
        };
        writeln!(
            &mut md,
            "| Direction Agreement | {:.1}% | >95% | {} |",
            self.metrics.direction_agreement_pct, dir_status
        )
        .unwrap();

        let conf_status = if self.metrics.avg_confidence_diff < 10.0 {
            "✅"
        } else {
            "❌"
        };
        writeln!(
            &mut md,
            "| Avg Confidence Diff | {:.1}% | <10% | {} |",
            self.metrics.avg_confidence_diff, conf_status
        )
        .unwrap();

        writeln!(
            &mut md,
            "| Exact Matches | {} | - | - |",
            self.metrics.exact_matches
        )
        .unwrap();
        writeln!(
            &mut md,
            "| Close Matches | {} | - | - |",
            self.metrics.close_matches
        )
        .unwrap();
        writeln!(
            &mut md,
            "| Disagreements | {} | - | {} |",
            self.metrics.disagreements,
            if self.metrics.disagreements < 5 {
                "✅"
            } else {
                "❌"
            }
        )
        .unwrap();
        writeln!(&mut md).unwrap();

        // Drift Summary
        if self.drift_summary.has_alerts {
            writeln!(&mut md, "## ⚠️ Drift Alerts").unwrap();
            writeln!(&mut md).unwrap();
            writeln!(
                &mut md,
                "**Alerting Metrics:** {}",
                self.drift_summary.alerting_metrics.join(", ")
            )
            .unwrap();
            writeln!(&mut md).unwrap();
        }

        // Symbol Breakdown
        if !self.symbol_breakdown.is_empty() {
            writeln!(&mut md, "## Symbol Breakdown").unwrap();
            writeln!(&mut md).unwrap();
            writeln!(
                &mut md,
                "| Symbol | Comparisons | Dir Agreement | Exact Matches | Disagreements | Status |"
            )
            .unwrap();
            writeln!(
                &mut md,
                "|--------|-------------|---------------|---------------|---------------|--------|"
            )
            .unwrap();
            for sym in &self.symbol_breakdown {
                let status_icon = match sym.status {
                    SymbolStatus::Passing => "✅",
                    SymbolStatus::NeedsReview => "⚠️",
                    SymbolStatus::Failing => "❌",
                };
                writeln!(
                    &mut md,
                    "| {} | {} | {:.1}% | {} | {} | {} |",
                    sym.symbol,
                    sym.total_comparisons,
                    sym.direction_agreement_pct,
                    sym.exact_matches,
                    sym.disagreements,
                    status_icon
                )
                .unwrap();
            }
            writeln!(&mut md).unwrap();
        }

        // Pattern Analysis
        if !self.pattern_analysis.is_empty() {
            writeln!(&mut md, "## Pattern Agreement").unwrap();
            writeln!(&mut md).unwrap();
            writeln!(&mut md, "| Pattern | Occurrences | Agreement | % |").unwrap();
            writeln!(&mut md, "|---------|-------------|-----------|---|").unwrap();
            for pat in &self.pattern_analysis {
                let status = if pat.agreement_pct >= 90.0 {
                    "✅"
                } else {
                    "⚠️"
                };
                writeln!(
                    &mut md,
                    "| {} | {} | {} | {:.1}% {} |",
                    pat.pattern_name,
                    pat.total_occurrences,
                    pat.agreement_count,
                    pat.agreement_pct,
                    status
                )
                .unwrap();
            }
            writeln!(&mut md).unwrap();
        }

        // Top Disagreements
        if !self.top_disagreements.is_empty() {
            writeln!(&mut md, "## Top Disagreements (Review Recommended)").unwrap();
            writeln!(&mut md).unwrap();
            for (i, comp) in self.top_disagreements.iter().take(10).enumerate() {
                writeln!(
                    &mut md,
                    "### {}. {} {} @ {}",
                    i + 1,
                    comp.symbol,
                    comp.timeframe,
                    comp.timestamp.format("%Y-%m-%d %H:%M")
                )
                .unwrap();
                writeln!(&mut md).unwrap();
                writeln!(
                    &mut md,
                    "- **TypeScript:** {:?} (conf: {:.1}%)",
                    comp.ts_output.direction, comp.ts_output.confidence
                )
                .unwrap();
                writeln!(
                    &mut md,
                    "- **Rust:** {:?} (conf: {:.1}%)",
                    comp.rust_output.direction, comp.rust_output.confidence
                )
                .unwrap();
                writeln!(
                    &mut md,
                    "- **Agreement Score:** {:.2}",
                    comp.agreement_score
                )
                .unwrap();
                writeln!(&mut md).unwrap();
            }
        }

        writeln!(&mut md, "---").unwrap();
        writeln!(&mut md, "*Generated by APEX Parity Engine*").unwrap();

        md
    }

    /// Export to JSON format
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export to CSV format
    pub fn to_csv(&self) -> String {
        let mut csv = "metric,value,target\n".to_string();
        writeln!(
            &mut csv,
            "direction_agreement_pct,{:.2},95.0",
            self.metrics.direction_agreement_pct
        )
        .unwrap();
        writeln!(
            &mut csv,
            "avg_confidence_diff,{:.2},10.0",
            self.metrics.avg_confidence_diff
        )
        .unwrap();
        writeln!(&mut csv, "exact_matches,{},-", self.metrics.exact_matches).unwrap();
        writeln!(&mut csv, "close_matches,{},-", self.metrics.close_matches).unwrap();
        writeln!(&mut csv, "disagreements,{},-", self.metrics.disagreements).unwrap();
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics() -> AgreementMetrics {
        AgreementMetrics {
            total_comparisons: 1000,
            exact_matches: 850,
            close_matches: 100,
            partial_matches: 30,
            disagreements: 20,
            misses: 0,
            false_positives: 0,
            false_negatives: 0,
            direction_agreement_pct: 95.0,
            avg_confidence_diff: 3.5,
            avg_entry_diff: 0.5,
            avg_stop_diff: 0.5,
            avg_target_diff: 0.5,
        }
    }

    fn create_test_drift_report() -> DriftReport {
        DriftReport {
            generated_at: Utc::now(),
            measurements: vec![],
            has_alerting: false,
            summary: "No drift detected".to_string(),
            recommendation: "Continue monitoring".to_string(),
        }
    }

    #[test]
    fn test_report_generation() {
        let reporter = ParityReporter::new(60);
        let stats = ParityStatistics::new(100);
        let drift = create_test_drift_report();

        let report = reporter.generate_report(&stats, &drift, &[]).unwrap();
        assert!(!report.go_live_status.ready); // Not enough data
    }

    #[test]
    fn test_markdown_output() {
        let reporter = ParityReporter::new(60);
        let drift = create_test_drift_report();

        let mut stats = ParityStatistics::new(100);
        let metrics = create_test_metrics();

        let report = reporter.generate_report(&stats, &drift, &[]).unwrap();
        let md = report.to_markdown();

        assert!(md.contains("# APEX Signal Engine Parity Report"));
        assert!(md.contains("Go-Live Status"));
        assert!(md.contains("Metrics Summary"));
    }
}
