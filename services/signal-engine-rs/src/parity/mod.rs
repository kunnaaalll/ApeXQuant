//! Parity Engine - Signal Comparison & Validation System
//!
//! This module provides comprehensive signal comparison between the Rust Signal Engine
//! and the TypeScript reference implementation. It tracks agreement, drift, and
//! statistical divergence between the two systems.
//!
//! # Go-Live Criteria
//! - Direction agreement > 95%
//! - Confidence drift < 10%
//! - Pattern agreement > 90%
//! - Zero panics, deterministic output

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod comparison;
pub mod drift;
pub mod reporter;
pub mod statistics;

pub use comparison::{ComparisonEngine, SignalComparison};
pub use drift::{DriftAnalyzer, DriftReport};
pub use reporter::{ParityReporter, ReportFormat};
pub use statistics::{ParityStatistics, StatisticsCollector};

/// Errors that can occur in parity operations
#[derive(Debug, Error)]
pub enum ParityError {
    #[error("Comparison failed: {0}")]
    ComparisonFailed(String),
    #[error("Statistics error: {0}")]
    StatisticsError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result type for parity operations
pub type Result<T> = std::result::Result<T, ParityError>;

/// Comparison classification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComparisonType {
    /// Signals are identical in all respects
    ExactMatch,
    /// Signals agree on direction with minor numerical differences
    CloseMatch,
    /// Signals partially agree (e.g., same direction but different patterns)
    PartialMatch,
    /// Signals disagree on direction
    Disagreement,
    /// Rust produced a signal, TypeScript did not
    Miss,
    /// TypeScript produced a signal, Rust did not (false positive candidate)
    FalsePositive,
    /// Rust did not produce a signal, TypeScript also did not
    FalseNegative,
}

/// Individual signal output from any engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOutput {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub timeframe: String,
    pub direction: SignalDirection,
    pub confidence: f64,
    pub confluence_score: f64,
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub patterns: Vec<String>,
    pub regime: String,
    pub session: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Signal direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalDirection {
    Long,
    Short,
    Neutral,
    NoSignal,
}

impl SignalDirection {
    /// Check if two directions agree (Long/Long or Short/Short)
    pub fn agrees_with(&self, other: &SignalDirection) -> bool {
        matches!(
            (self, other),
            (SignalDirection::Long, SignalDirection::Long)
                | (SignalDirection::Short, SignalDirection::Short)
        )
    }

    /// Check if direction is actionable (Long or Short)
    pub fn is_actionable(&self) -> bool {
        matches!(self, SignalDirection::Long | SignalDirection::Short)
    }
}

/// Agreement metrics for a comparison batch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgreementMetrics {
    pub total_comparisons: u64,
    pub exact_matches: u64,
    pub close_matches: u64,
    pub partial_matches: u64,
    pub disagreements: u64,
    pub misses: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
    pub direction_agreement_pct: f64,
    pub avg_confidence_diff: f64,
    pub avg_entry_diff: f64,
    pub avg_stop_diff: f64,
    pub avg_target_diff: f64,
}

impl AgreementMetrics {
    /// Calculate direction agreement percentage
    pub fn calculate_direction_agreement(&mut self) {
        let agreeing = self.exact_matches + self.close_matches;
        if self.total_comparisons > 0 {
            self.direction_agreement_pct = (agreeing as f64 / self.total_comparisons as f64) * 100.0;
        }
    }

    /// Check if go-live criteria are met
    pub fn meets_go_live_criteria(&self) -> bool {
        self.direction_agreement_pct > 95.0
            && self.avg_confidence_diff < 10.0
    }
}

/// Pattern comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternComparison {
    pub pattern_name: String,
    pub rust_detected: bool,
    pub ts_detected: bool,
    pub rust_strength: f64,
    pub ts_strength: f64,
    pub agreement: bool,
}

/// Regime comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeComparison {
    pub rust_regime: String,
    pub ts_regime: String,
    pub agreement: bool,
}

/// Complete comparison for a single signal opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalComparisonRecord {
    pub comparison_id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub timeframe: String,
    pub ts_output: SignalOutput,
    pub rust_output: SignalOutput,
    pub comparison_type: ComparisonType,
    pub direction_match: bool,
    pub confidence_diff: f64,
    pub entry_diff: Option<f64>,
    pub stop_diff: Option<f64>,
    pub target_diff: Option<f64>,
    pub pattern_comparisons: Vec<PatternComparison>,
    pub regime_comparison: RegimeComparison,
    pub agreement_score: f64,
    pub notes: String,
}

/// Configuration for the parity system
#[derive(Debug, Clone)]
pub struct ParityConfig {
    pub close_match_threshold: f64,
    pub price_tolerance_pips: f64,
    pub confidence_tolerance: f64,
    pub enable_storage: bool,
    pub storage_path: Option<String>,
    pub report_interval_minutes: u64,
}

impl Default for ParityConfig {
    fn default() -> Self {
        Self {
            close_match_threshold: 0.9,
            price_tolerance_pips: 5.0,
            confidence_tolerance: 5.0,
            enable_storage: true,
            storage_path: None,
            report_interval_minutes: 60,
        }
    }
}

/// Main parity orchestrator
pub struct ParityEngine {
    config: ParityConfig,
    comparison_engine: ComparisonEngine,
    statistics: StatisticsCollector,
    drift_analyzer: DriftAnalyzer,
    reporter: ParityReporter,
}

impl ParityEngine {
    /// Create a new parity engine with the given configuration
    pub fn new(config: ParityConfig) -> Self {
        let comparison_engine = ComparisonEngine::new(&config);
        let statistics = StatisticsCollector::new();
        let drift_analyzer = DriftAnalyzer::new();
        let reporter = ParityReporter::new(config.report_interval_minutes);

        Self {
            config,
            comparison_engine,
            statistics,
            drift_analyzer,
            reporter,
        }
    }

    /// Compare a pair of signals and record the result
    pub fn compare_signals(
        &mut self,
        ts_output: SignalOutput,
        rust_output: SignalOutput,
    ) -> Result<SignalComparisonRecord> {
        let record = self
            .comparison_engine
            .compare(ts_output, rust_output)?;

        self.statistics.record(&record)?;
        self.drift_analyzer.record(&record)?;

        Ok(record)
    }

    /// Get current agreement metrics
    pub fn get_metrics(&self) -> AgreementMetrics {
        self.statistics.get_metrics()
    }

    /// Generate a drift report
    pub fn generate_drift_report(&self) -> Result<DriftReport> {
        self.drift_analyzer.generate_report()
    }

    /// Check if system meets go-live criteria
    pub fn can_go_live(&self) -> bool {
        let metrics = self.get_metrics();
        metrics.meets_go_live_criteria()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_agreement() {
        assert!(SignalDirection::Long.agrees_with(&SignalDirection::Long));
        assert!(SignalDirection::Short.agrees_with(&SignalDirection::Short));
        assert!(!SignalDirection::Long.agrees_with(&SignalDirection::Short));
        assert!(!SignalDirection::Short.agrees_with(&SignalDirection::Long));
    }

    #[test]
    fn test_agreement_metrics_go_live() {
        let metrics = AgreementMetrics {
            total_comparisons: 100,
            exact_matches: 90,
            close_matches: 5,
            direction_agreement_pct: 95.0,
            avg_confidence_diff: 5.0,
            ..Default::default()
        };

        assert!(metrics.meets_go_live_criteria());
    }
}
