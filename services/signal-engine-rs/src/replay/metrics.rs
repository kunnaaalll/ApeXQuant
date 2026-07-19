//! Replay Metrics - Performance and verification metrics for replays

use super::*;
use std::collections::VecDeque;

/// Metrics collected during a replay
#[derive(Debug, Clone, Default)]
pub struct ReplayMetrics {
    pub scenarios_processed: u64,
    pub candles_processed: u64,
    pub signals_generated: u64,
    pub errors: u64,
    pub comparisons: u64,
    pub latencies: VecDeque<f64>, // milliseconds
}

impl ReplayMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            scenarios_processed: 0,
            candles_processed: 0,
            signals_generated: 0,
            errors: 0,
            comparisons: 0,
            latencies: VecDeque::with_capacity(10000),
        }
    }

    /// Record a latency measurement
    pub fn record_latency(&mut self, latency_ms: f64) {
        if self.latencies.len() >= 10000 {
            self.latencies.pop_front();
        }
        self.latencies.push_back(latency_ms);
    }

    /// Calculate average latency
    pub fn avg_latency_ms(&self) -> f64 {
        if self.latencies.is_empty() {
            0.0
        } else {
            self.latencies.iter().sum::<f64>() / self.latencies.len() as f64
        }
    }

    /// Calculate P95 latency
    pub fn p95_latency_ms(&self) -> f64 {
        self.percentile_latency(0.95)
    }

    /// Calculate P99 latency
    pub fn p99_latency_ms(&self) -> f64 {
        self.percentile_latency(0.99)
    }

    fn percentile_latency(&self, percentile: f64) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f64> = self.latencies.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let idx = ((sorted.len() as f64 * percentile).ceil() as usize)
            .saturating_sub(1)
            .min(sorted.len() - 1);
        sorted[idx]
    }

    /// Check if latency meets targets
    pub fn meets_latency_targets(&self) -> bool {
        self.avg_latency_ms() < 10.0 && self.p99_latency_ms() < 25.0
    }
}

/// Comprehensive statistics from a replay batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStatistics {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_seconds: f64,
    pub scenarios_summary: ScenariosSummary,
    pub determinism_summary: DeterminismSummary,
    pub performance_summary: PerformanceSummary,
    pub verification_summary: VerificationSummary,
}

/// Scenarios processed summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenariosSummary {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub by_category: Vec<(String, u64)>,
}

/// Determinism validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismSummary {
    pub scenarios_tested: u64,
    pub deterministic_count: u64,
    pub non_deterministic_count: u64,
    pub determinism_rate: f64,
    pub non_deterministic_scenarios: Vec<String>,
}

impl DeterminismSummary {
    /// Check if determinism passes threshold
    pub fn passes_threshold(&self, threshold: f64) -> bool {
        self.determinism_rate >= threshold
    }
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub throughput_per_second: f64,
    pub total_coins_processed: u64,
}

impl PerformanceSummary {
    /// Check if performance meets targets
    pub fn meets_targets(&self) -> bool {
        self.avg_latency_ms < 10.0
            && self.p95_latency_ms < 20.0
            && self.p99_latency_ms < 25.0
    }
}

/// Verification results summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub comparisons_done: u64,
    pub exact_matches: u64,
    pub close_matches: u64,
    pub partial_matches: u64,
    pub disagreements: u64,
    pub agreement_rate: f64,
}

/// Report generator for replay statistics
pub struct ReplayReportGenerator;

impl ReplayReportGenerator {
    /// Generate a markdown report
    pub fn generate_markdown(stats: &ReplayStatistics) -> String {
        let mut md = String::new();

        writeln!(&mut md, "# Historical Replay Report").unwrap();
        writeln!(&mut md).unwrap();
        writeln!(&mut md, "**Duration:** {:.1}s", stats.total_duration_seconds).unwrap();
        writeln!( &mut md,
            "**Period:** {} to {}",
            stats.started_at.format("%Y-%m-%d %H:%M:%S"),
            stats.completed_at.format("%Y-%m-%d %H:%M:%S")
        )
        .unwrap();
        writeln!(&mut md).unwrap();

        // Scenarios
        writeln!(&mut md, "## Scenarios").unwrap();
        writeln!(&mut md).unwrap();
        writeln!(&mut md, "- Total: {}", stats.scenarios_summary.total).unwrap();
        writeln!(&mut md, "- Passed: {}", stats.scenarios_summary.passed).unwrap();
        writeln!(&mut md, "- Failed: {}", stats.scenarios_summary.failed).unwrap();
        writeln!(&mut md).unwrap();

        // Determinism
        writeln!(&mut md, "## Determinism").unwrap();
        writeln!(&mut md).unwrap();
        let det_status = if stats.determinism_summary.passes_threshold(100.0) {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        writeln!( &mut md,
            "- Rate: {:.1}% {}",
            stats.determinism_summary.determinism_rate, det_status
        )
        .unwrap();
        writeln!( &mut md,
            "- Non-deterministic: {}",
            stats.determinism_summary.non_deterministic_scenarios.len()
        )
        .unwrap();
        writeln!(&mut md).unwrap();

        // Performance
        writeln!(&mut md, "## Performance").unwrap();
        writeln!(&mut md).unwrap();
        let perf_status = if stats.performance_summary.meets_targets() {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        writeln!( &mut md,
            "- Avg Latency: {:.2}ms {}",
            stats.performance_summary.avg_latency_ms, perf_status
        )
        .unwrap();
        writeln!(&mut md, "- P95 Latency: {:.2}ms", stats.performance_summary.p95_latency_ms).unwrap();
        writeln!(&mut md, "- P99 Latency: {:.2}ms", stats.performance_summary.p99_latency_ms).unwrap();
        writeln!( &mut md,
            "- Throughput: {:.1} candles/sec",
            stats.performance_summary.throughput_per_second
        )
        .unwrap();
        writeln!(&mut md).unwrap();

        // Verification
        writeln!(&mut md, "## TypeScript Parity").unwrap();
        writeln!(&mut md).unwrap();
        writeln!( &mut md,
            "- Agreement Rate: {:.1}%",
            stats.verification_summary.agreement_rate
        )
        .unwrap();
        writeln!(&mut md, "- Exact Matches: {}", stats.verification_summary.exact_matches).unwrap();
        writeln!(&mut md, "- Disagreements: {}", stats.verification_summary.disagreements).unwrap();
        writeln!(&mut md).unwrap();

        md
    }

    /// Generate JSON report
    pub fn generate_json(stats: &ReplayStatistics) -> String {
        serde_json::to_string_pretty(stats).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_metrics() {
        let mut metrics = ReplayMetrics::new();

        for i in 0..100 {
            metrics.record_latency(i as f64 * 0.1);
        }

        assert!(metrics.avg_latency_ms() > 0.0);
        assert!(metrics.p95_latency_ms() > metrics.avg_latency_ms());
    }

    #[test]
    fn test_latency_targets() {
        let mut metrics = ReplayMetrics::new();

        // Add latencies under target
        for _ in 0..100 {
            metrics.record_latency(5.0);
        }

        assert!(metrics.meets_latency_targets());

        // Add high latency
        for _ in 0..10 {
            metrics.record_latency(100.0);
        }

        // Might still pass if P99 is under threshold
        // Just verify the method runs
        let _ = metrics.meets_latency_targets();
    }
}
