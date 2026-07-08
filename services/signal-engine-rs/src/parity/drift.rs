//! Drift Analyzer - Detect confidence and behavior drift over time
//!
//! Monitors for systematic deviations between TypeScript and Rust engines,
//! alerting when drift exceeds acceptable thresholds.

use super::*;
use std::collections::VecDeque;

/// Drift detection configuration
#[derive(Debug, Clone)]
pub struct DriftConfig {
    /// Threshold for confidence drift alarm (% difference)
    pub confidence_drift_threshold: f64,
    /// Threshold for entry price drift alarm (pips)
    pub entry_drift_threshold: f64,
    /// Threshold for pattern detection drift alarm (% difference)
    pub pattern_drift_threshold: f64,
    /// Minimum samples before drift can trigger
    pub min_samples: usize,
    /// Window size for drift calculation
    pub window_size: usize,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            confidence_drift_threshold: 10.0, // 10%
            entry_drift_threshold: 10.0,      // 10 pips
            pattern_drift_threshold: 15.0,    // 15%
            min_samples: 100,
            window_size: 500,
        }
    }
}

/// Drift measurement for a specific metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMeasurement {
    pub metric_name: String,
    pub ts_mean: f64,
    pub rust_mean: f64,
    pub absolute_drift: f64,
    pub relative_drift_pct: f64,
    pub drift_per_hour: f64,
    pub trend: DriftTrend,
    pub is_alerting: bool,
}

/// Direction of drift trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftTrend {
    /// Drift is stable within normal bounds
    Stable,
    /// Rust values are trending higher than TypeScript
    RustHigher,
    /// Rust values are trending lower than TypeScript
    RustLower,
    /// Drift is oscillating but not trending
    Oscillating,
    /// Not enough data to determine
    InsufficientData,
}

/// A single drift data point
#[derive(Debug, Clone)]
struct DriftPoint {
    timestamp: DateTime<Utc>,
    ts_value: f64,
    rust_value: f64,
    diff: f64,
}

/// Time-series drift tracker for a single metric
pub struct TimeSeriesDriftTracker {
    metric_name: String,
    points: VecDeque<DriftPoint>,
    window_size: usize,
    threshold: f64,
}

impl TimeSeriesDriftTracker {
    /// Create a new drift tracker
    pub fn new(metric_name: String, window_size: usize, threshold: f64) -> Self {
        Self {
            metric_name,
            points: VecDeque::with_capacity(window_size),
            window_size,
            threshold,
        }
    }

    /// Record a new data point
    pub fn record(&mut self, ts_value: f64, rust_value: f64) {
        let point = DriftPoint {
            timestamp: Utc::now(),
            ts_value,
            rust_value,
            diff: (rust_value - ts_value).abs(),
        };

        if self.points.len() >= self.window_size {
            self.points.pop_front();
        }
        self.points.push_back(point);
    }

    /// Calculate current drift measurement
    pub fn measure(&self, min_samples: usize) -> DriftMeasurement {
        if self.points.len() < min_samples {
            return DriftMeasurement {
                metric_name: self.metric_name.clone(),
                ts_mean: 0.0,
                rust_mean: 0.0,
                absolute_drift: 0.0,
                relative_drift_pct: 0.0,
                drift_per_hour: 0.0,
                trend: DriftTrend::InsufficientData,
                is_alerting: false,
            };
        }

        let ts_mean =
            self.points.iter().map(|p| p.ts_value).sum::<f64>() / self.points.len() as f64;
        let rust_mean =
            self.points.iter().map(|p| p.rust_value).sum::<f64>() / self.points.len() as f64;

        let absolute_drift = (rust_mean - ts_mean).abs();
        let relative_drift_pct = if ts_mean.abs() > 0.0001 {
            (absolute_drift / ts_mean.abs()) * 100.0
        } else {
            0.0
        };

        // Calculate drift per hour
        let drift_per_hour = self.calculate_drift_rate();

        let trend = self.analyze_trend();
        let is_alerting = absolute_drift > self.threshold || relative_drift_pct > self.threshold;

        DriftMeasurement {
            metric_name: self.metric_name.clone(),
            ts_mean,
            rust_mean,
            absolute_drift,
            relative_drift_pct,
            drift_per_hour,
            trend,
            is_alerting,
        }
    }

    /// Calculate drift rate per hour
    fn calculate_drift_rate(&self) -> f64 {
        if self.points.len() < 10 {
            return 0.0;
        }

        let first = self.points.front().unwrap();
        let last = self.points.back().unwrap();
        let duration_hours = last
            .timestamp
            .signed_duration_since(first.timestamp)
            .num_seconds() as f64
            / 3600.0;

        if duration_hours <= 0.0 {
            return 0.0;
        }

        (last.diff - first.diff).abs() / duration_hours
    }

    /// Analyze trend direction
    fn analyze_trend(&self) -> DriftTrend {
        if self.points.len() < 20 {
            return DriftTrend::InsufficientData;
        }

        // Split into halves and compare
        let half = self.points.len() / 2;
        let first_half_mean: f64 = self
            .points
            .iter()
            .take(half)
            .map(|p| p.rust_value - p.ts_value)
            .sum::<f64>()
            / half as f64;

        let second_half_mean: f64 = self
            .points
            .iter()
            .skip(half)
            .map(|p| p.rust_value - p.ts_value)
            .sum::<f64>()
            / (self.points.len() - half) as f64;

        let diff = second_half_mean - first_half_mean;
        let variance: f64 = self
            .points
            .iter()
            .map(|p| {
                let dev = (p.rust_value - p.ts_value) - first_half_mean;
                dev * dev
            })
            .sum::<f64>()
            / self.points.len() as f64;

        let std_dev = variance.sqrt();

        if diff.abs() < std_dev {
            if std_dev < self.threshold {
                DriftTrend::Stable
            } else {
                DriftTrend::Oscillating
            }
        } else if diff > 0.0 {
            DriftTrend::RustHigher
        } else {
            DriftTrend::RustLower
        }
    }
}

/// Drift analyzer for all metrics
pub struct DriftAnalyzer {
    config: DriftConfig,
    confidence_tracker: TimeSeriesDriftTracker,
    entry_tracker: TimeSeriesDriftTracker,
    stop_tracker: TimeSeriesDriftTracker,
    target_tracker: TimeSeriesDriftTracker,
    pattern_tracker: TimeSeriesDriftTracker,
    regime_tracker: TimeSeriesDriftTracker,
}

impl DriftAnalyzer {
    /// Create a new drift analyzer
    pub fn new() -> Self {
        let config = DriftConfig::default();
        let window_size = config.window_size;

        Self {
            confidence_tracker: TimeSeriesDriftTracker::new(
                "confidence".to_string(),
                window_size,
                config.confidence_drift_threshold,
            ),
            entry_tracker: TimeSeriesDriftTracker::new(
                "entry_price".to_string(),
                window_size,
                config.entry_drift_threshold,
            ),
            stop_tracker: TimeSeriesDriftTracker::new(
                "stop_loss".to_string(),
                window_size,
                config.entry_drift_threshold,
            ),
            target_tracker: TimeSeriesDriftTracker::new(
                "take_profit".to_string(),
                window_size,
                config.entry_drift_threshold,
            ),
            pattern_tracker: TimeSeriesDriftTracker::new(
                "pattern_agreement".to_string(),
                window_size,
                config.pattern_drift_threshold,
            ),
            regime_tracker: TimeSeriesDriftTracker::new(
                "regime_agreement".to_string(),
                window_size,
                config.pattern_drift_threshold,
            ),
            config,
        }
    }

    /// Record a comparison
    pub fn record(&mut self, comparison: &SignalComparisonRecord) -> Result<()> {
        // Record confidence values
        self.confidence_tracker.record(
            comparison.ts_output.confidence,
            comparison.rust_output.confidence,
        );

        // Record price values if available
        if let (Some(ts_entry), Some(rust_entry)) = (
            comparison.ts_output.entry_price,
            comparison.rust_output.entry_price,
        ) {
            self.entry_tracker.record(ts_entry, rust_entry);
        }

        if let (Some(ts_sl), Some(rust_sl)) = (
            comparison.ts_output.stop_loss,
            comparison.rust_output.stop_loss,
        ) {
            self.stop_tracker.record(ts_sl, rust_sl);
        }

        if let (Some(ts_tp), Some(rust_tp)) = (
            comparison.ts_output.take_profit,
            comparison.rust_output.take_profit,
        ) {
            self.target_tracker.record(ts_tp, rust_tp);
        }

        // Pattern agreement percentage
        let pattern_agreement_pct = if comparison.pattern_comparisons.is_empty() {
            100.0
        } else {
            let agreed = comparison
                .pattern_comparisons
                .iter()
                .filter(|p| p.agreement)
                .count();
            (agreed as f64 / comparison.pattern_comparisons.len() as f64) * 100.0
        };
        self.pattern_tracker.record(100.0, pattern_agreement_pct);

        // Regime agreement
        let regime_agreement = if comparison.regime_comparison.agreement {
            100.0
        } else {
            0.0
        };
        self.regime_tracker.record(100.0, regime_agreement);

        Ok(())
    }

    /// Generate a complete drift report
    pub fn generate_report(&self) -> Result<DriftReport> {
        let measurements = vec![
            self.confidence_tracker.measure(self.config.min_samples),
            self.entry_tracker.measure(self.config.min_samples),
            self.stop_tracker.measure(self.config.min_samples),
            self.target_tracker.measure(self.config.min_samples),
            self.pattern_tracker.measure(self.config.min_samples),
            self.regime_tracker.measure(self.config.min_samples),
        ];

        let alerting_count = measurements.iter().filter(|m| m.is_alerting).count();
        let has_alerting = alerting_count > 0;

        let summary = format!(
            "Drift analysis: {} of {} metrics alerting. Confidence drift: {:.2}%, Pattern drift: {:.2}%",
            alerting_count,
            measurements.len(),
            measurements[0].relative_drift_pct,
            measurements[4].relative_drift_pct
        );

        Ok(DriftReport {
            generated_at: Utc::now(),
            measurements: measurements.clone(),
            has_alerting,
            summary,
            recommendation: self.generate_recommendation(&measurements, has_alerting),
        })
    }

    /// Generate recommendation based on drift status
    fn generate_recommendation(
        &self,
        measurements: &[DriftMeasurement],
        has_alerting: bool,
    ) -> String {
        if !has_alerting {
            return "Drift within acceptable bounds. Continue monitoring.".to_string();
        }

        let mut issues = Vec::new();

        for m in measurements {
            if m.is_alerting {
                issues.push(format!(
                    "{} drift: {:.2}% ({:?})",
                    m.metric_name, m.relative_drift_pct, m.trend
                ));
            }
        }

        if issues.is_empty() {
            return "Drift within acceptable bounds. Continue monitoring.".to_string();
        }

        format!(
            "DRIFT ALERT: {}. Review signal generation logic for these metrics.",
            issues.join(", ")
        )
    }

    /// Check if any metric is alerting
    pub fn has_alerts(&self) -> bool {
        self.generate_report().map_or(false, |r| r.has_alerting)
    }
}

impl Default for DriftAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete drift report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub generated_at: DateTime<Utc>,
    pub measurements: Vec<DriftMeasurement>,
    pub has_alerting: bool,
    pub summary: String,
    pub recommendation: String,
}

impl DriftReport {
    /// Check if confidence drift is within go-live criteria
    pub fn meets_go_live_criteria(&self) -> bool {
        self.measurements
            .iter()
            .find(|m| m.metric_name == "confidence")
            .map_or(false, |m| m.relative_drift_pct < 10.0 && !m.is_alerting)
    }

    /// Get alerting metrics
    pub fn alerting_metrics(&self) -> Vec<&DriftMeasurement> {
        self.measurements.iter().filter(|m| m.is_alerting).collect()
    }

    /// Generate markdown summary
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# Drift Report\n\nGenerated: {}\n\n## Summary\n\n{}\n\n## Measurements\n\n| Metric | TS Mean | Rust Mean | Drift % | Trend | Status |\n|--------|---------|-----------|---------|-------|--------|\n",
            self.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            self.summary
        );

        for m in &self.measurements {
            let trend = format!("{:?}", m.trend);
            let status = if m.is_alerting { "ALERT" } else { "OK" };
            md.push_str(&format!(
                "| {} | {:.2} | {:.2} | {:.2}% | {} | {} |\n",
                m.metric_name, m.ts_mean, m.rust_mean, m.relative_drift_pct, trend, status
            ));
        }

        md.push_str(&format!("\n## Recommendation\n\n{}\n", self.recommendation));

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_tracker_basic() {
        let mut tracker = TimeSeriesDriftTracker::new("test".to_string(), 100, 10.0);

        // Record some points
        for i in 0..20 {
            tracker.record(100.0, 102.0); // 2% drift
        }

        let measurement = tracker.measure(10);
        assert!(!measurement.is_alerting);
        assert_eq!(measurement.relative_drift_pct, 2.0);
    }

    #[test]
    fn test_drift_tracker_alert() {
        let mut tracker = TimeSeriesDriftTracker::new("test".to_string(), 100, 5.0);

        // Record points with high drift
        for i in 0..20 {
            tracker.record(100.0, 110.0); // 10% drift
        }

        let measurement = tracker.measure(10);
        assert!(measurement.is_alerting);
        assert_eq!(measurement.relative_drift_pct, 10.0);
    }

    #[test]
    fn test_trend_analysis() {
        let mut tracker = TimeSeriesDriftTracker::new("test".to_string(), 100, 10.0);

        // First half: small positive drift
        for i in 0..30 {
            tracker.record(100.0, 101.0);
        }

        // Second half: larger positive drift (trending higher)
        for i in 0..30 {
            tracker.record(100.0, 105.0);
        }

        let measurement = tracker.measure(20);
        assert_eq!(measurement.trend, DriftTrend::RustHigher);
    }
}
