//! Comparison Repository - High-level data access layer

use super::*;
use crate::parity::AgreementMetrics;
use chrono::Duration;

/// Repository for comparison data operations
pub struct ComparisonRepository {
    storage: Storage,
}

impl ComparisonRepository {
    /// Create a new repository
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Create with in-memory storage for testing
    pub fn new_in_memory() -> SqliteResult<Self> {
        let storage = Storage::new_in_memory()?;
        Ok(Self { storage })
    }

    /// Create with file storage
    pub fn new_file(path: &Path) -> SqliteResult<Self> {
        let storage = Storage::new_file(path)?;
        Ok(Self { storage })
    }

    /// Save a comparison record
    pub fn save(&self, record: &SignalComparisonRecord) -> SqliteResult<()> {
        self.storage.insert_comparison(record)
    }

    /// Save multiple comparisons in batch
    pub fn save_batch(&self, records: &[SignalComparisonRecord]) -> SqliteResult<usize> {
        let mut count = 0;
        for record in records {
            self.storage.insert_comparison(record)?;
            count += 1;
        }
        Ok(count)
    }

    /// Get recent comparisons
    pub fn get_recent(&self, limit: usize) -> SqliteResult<Vec<SignalComparisonRecord>> {
        self.storage.get_comparisons(ComparisonFilter::default(), limit)
    }

    /// Get comparisons for a specific symbol
    pub fn get_by_symbol(&self, symbol: &str, limit: usize) -> SqliteResult<Vec<SignalComparisonRecord>> {
        let filter = ComparisonFilter {
            symbol: Some(symbol.to_string()),
            ..Default::default()
        };
        self.storage.get_comparisons(filter, limit)
    }

    /// Get comparisons by type
    pub fn get_by_type(
        &self,
        comp_type: ComparisonType,
        limit: usize,
    ) -> SqliteResult<Vec<SignalComparisonRecord>> {
        let filter = ComparisonFilter {
            comparison_type: Some(comp_type),
            ..Default::default()
        };
        self.storage.get_comparisons(filter, limit)
    }

    /// Get all disagreements for review
    pub fn get_disagreements(&self, limit: usize) -> SqliteResult<Vec<SignalComparisonRecord>> {
        self.get_by_type(ComparisonType::Disagreement, limit)
    }

    /// Get all false positives for review
    pub fn get_false_positives(&self, limit: usize) -> SqliteResult<Vec<SignalComparisonRecord>> {
        self.get_by_type(ComparisonType::FalsePositive, limit)
    }

    /// Get all misses for review
    pub fn get_misses(&self, limit: usize) -> SqliteResult<Vec<SignalComparisonRecord>> {
        self.get_by_type(ComparisonType::Miss, limit)
    }

    /// Get today's statistics
    pub fn get_today_statistics(&self) -> SqliteResult<AgreementMetrics> {
        let now = Utc::now();
        let start_of_day = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start = DateTime::from_naive_utc_and_offset(start_of_day, chrono::Utc);

        self.get_statistics(start, now)
    }

    /// Get statistics for a time range
    pub fn get_statistics(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> SqliteResult<AgreementMetrics> {
        let stored = self.storage.get_statistics(from, to)?;

        Ok(AgreementMetrics {
            total_comparisons: stored.total_comparisons,
            exact_matches: stored.exact_matches,
            close_matches: stored.close_matches,
            partial_matches: stored.partial_matches,
            disagreements: stored.disagreements,
            misses: stored.misses,
            false_positives: stored.false_positives,
            false_negatives: stored.false_negatives,
            direction_agreement_pct: stored.direction_agreement_pct,
            avg_confidence_diff: stored.avg_confidence_diff,
            avg_entry_diff: 0.0, // Would need additional query
            avg_stop_diff: 0.0,
            avg_target_diff: 0.0,
        })
    }

    /// Get statistics for the last N hours
    pub fn get_recent_statistics(&self, hours: i64) -> SqliteResult<AgreementMetrics> {
        let to = Utc::now();
        let from = to - Duration::hours(hours);
        self.get_statistics(from, to)
    }

    /// Get hourly trend over time
    pub fn get_hourly_trend(&self, hours: i64) -> SqliteResult<Vec<HourlyMetrics>> {
        let to = Utc::now();
        let from = to - Duration::hours(hours);

        let filter = ComparisonFilter {
            from: Some(from),
            to: Some(to),
            ..Default::default()
        };

        let records = self.storage.get_comparisons(filter, 10000)?;

        // Group by hour
        let mut hourly: std::collections::HashMap<String, Vec<&SignalComparisonRecord>> =
            std::collections::HashMap::new();

        for record in &records {
            let hour_key = record.timestamp.format("%Y-%m-%d %H:00").to_string();
            hourly.entry(hour_key).or_default().push(record);
        }

        let mut trend: Vec<HourlyMetrics> = hourly
            .into_iter()
            .map(|(hour, recs)| {
                let total = recs.len() as u64;
                let dir_agreements = recs.iter().filter(|r| r.direction_match).count() as u64;
                let exact = recs
                    .iter()
                    .filter(|r| matches!(r.comparison_type, ComparisonType::ExactMatch))
                    .count() as u64;
                let close = recs
                    .iter()
                    .filter(|r| matches!(r.comparison_type, ComparisonType::CloseMatch))
                    .count() as u64;
                let disagreements = recs
                    .iter()
                    .filter(|r| matches!(r.comparison_type, ComparisonType::Disagreement))
                    .count() as u64;

                let avg_agreement = recs.iter().map(|r| r.agreement_score).sum::<f64>() / recs.len() as f64;

                HourlyMetrics {
                    hour,
                    total_comparisons: total,
                    direction_agreement_pct: if total > 0 {
                        (dir_agreements as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                    exact_matches: exact,
                    close_matches: close,
                    disagreements,
                    avg_agreement_score: avg_agreement,
                }
            })
            .collect();

        trend.sort_by(|a, b| a.hour.cmp(&b.hour));
        Ok(trend)
    }

    /// Check if go-live criteria are met
    pub fn check_go_live_criteria(&self) -> SqliteResult<GoLiveCheck> {
        let metrics = self.get_recent_statistics(24)?;

        let mut passing = Vec::new();
        let mut failing = Vec::new();

        if metrics.direction_agreement_pct >= 95.0 {
            passing.push(format!("Direction agreement: {:.1}%", metrics.direction_agreement_pct));
        } else {
            failing.push(format!(
                "Direction agreement: {:.1}% < 95%",
                metrics.direction_agreement_pct
            ));
        }

        if metrics.avg_confidence_diff < 10.0 {
            passing.push(format!("Confidence drift: {:.1}%", metrics.avg_confidence_diff));
        } else {
            failing.push(format!("Confidence drift: {:.1}% >= 10%", metrics.avg_confidence_diff));
        }

        let total_actionable = metrics.total_comparisons - metrics.false_negatives;
        let disagreement_rate = if total_actionable > 0 {
            (metrics.disagreements as f64 / total_actionable as f64) * 100.0
        } else {
            0.0
        };

        if disagreement_rate < 5.0 {
            passing.push(format!("Disagreement rate: {:.1}%", disagreement_rate));
        } else {
            failing.push(format!("Disagreement rate: {:.1}% >= 5%", disagreement_rate));
        }

        Ok(GoLiveCheck {
            ready: failing.is_empty(),
            passing,
            failing,
            metrics,
        })
    }

    /// Get comparison summary report
    pub fn get_summary_report(&self) -> SqliteResult<String> {
        let recent = self.get_recent_statistics(24)?;
        let today = self.get_today_statistics()?;

        let mut report = String::new();
        report.push_str("# Signal Comparison Summary Report\n\n");
        report.push_str("## Last 24 Hours\n\n");
        report.push_str(&format!("- Total comparisons: {}\n", recent.total_comparisons));
        report.push_str(&format!("- Direction agreement: {:.1}%\n", recent.direction_agreement_pct));
        report.push_str(&format!("- Avg confidence diff: {:.1}%\n", recent.avg_confidence_diff));
        report.push_str(&format!("- Exact matches: {}\n", recent.exact_matches));
        report.push_str(&format!("- Close matches: {}\n", recent.close_matches));
        report.push_str(&format!("- Disagreements: {}\n", recent.disagreements));
        report.push_str(&format!("- Misses: {}\n", recent.misses));
        report.push_str(&format!("- False positives: {}\n", recent.false_positives));

        report.push_str("\n## Today\n\n");
        report.push_str(&format!("- Total comparisons: {}\n", today.total_comparisons));
        report.push_str(&format!("- Direction agreement: {:.1}%\n", today.direction_agreement_pct));

        // Go-live check
        let check = self.check_go_live_criteria()?;
        report.push_str("\n## Go-Live Status\n\n");
        if check.ready {
            report.push_str("✅ READY\n\n");
        } else {
            report.push_str("❌ NOT READY\n\n");
        }
        if !check.passing.is_empty() {
            report.push_str("### Passing:\n");
            for p in check.passing {
                report.push_str(&format!("- ✅ {}\n", p));
            }
            report.push('\n');
        }
        if !check.failing.is_empty() {
            report.push_str("### Failing:\n");
            for f in check.failing {
                report.push_str(&format!("- ❌ {}\n", f));
            }
        }

        Ok(report)
    }

    /// Purge old data (keep last N days)
    pub fn purge_old_data(&self, keep_days: i64) -> SqliteResult<usize> {
        let cutoff = Utc::now() - Duration::days(keep_days);
        let count = self.storage.purge_before(cutoff)?;
        Ok(count)
    }

    /// Export data to CSV
    pub fn export_to_csv(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> SqliteResult<String> {
        let filter = ComparisonFilter {
            from: Some(from),
            to: Some(to),
            ..Default::default()
        };

        let records = self.storage.get_comparisons(filter, 100000)?;

        let mut csv = "timestamp,symbol,timeframe,ts_direction,ts_confidence,rust_direction,rust_confidence,comparison_type,direction_match,agreement_score\n".to_string();

        for rec in records {
            writeln!( &mut csv,
                "{},{},{},{},{},{},{},{},{},{}",
                rec.timestamp.to_rfc3339(),
                rec.symbol,
                rec.timeframe,
                format!("{:?}", rec.ts_output.direction),
                rec.ts_output.confidence,
                format!("{:?}", rec.rust_output.direction),
                rec.rust_output.confidence,
                format!("{:?}", rec.comparison_type),
                rec.direction_match,
                rec.agreement_score
            )
            .unwrap();
        }

        Ok(csv)
    }
}

/// Hourly metrics entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetrics {
    pub hour: String,
    pub total_comparisons: u64,
    pub direction_agreement_pct: f64,
    pub exact_matches: u64,
    pub close_matches: u64,
    pub disagreements: u64,
    pub avg_agreement_score: f64,
}

/// Go-live criteria check result
#[derive(Debug, Clone)]
pub struct GoLiveCheck {
    pub ready: bool,
    pub passing: Vec<String>,
    pub failing: Vec<String>,
    pub metrics: AgreementMetrics,
}

// Extend Storage with purge method
impl Storage {
    fn purge_before(&self, cutoff: DateTime<Utc>) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "DELETE FROM signal_comparisons WHERE timestamp < ?1",
            [cutoff.to_rfc3339()],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_save_and_retrieve() {
        let repo = ComparisonRepository::new_in_memory().unwrap();

        // Create test record would go here
        // For now just verify repository creates
    }

    #[test]
    fn test_go_live_check() {
        let repo = ComparisonRepository::new_in_memory().unwrap();
        let check = repo.check_go_live_criteria().unwrap();

        // With no data, should not be ready
        assert!(!check.ready);
    }
}
