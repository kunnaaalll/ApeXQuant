//! Comparison Engine - Core signal comparison logic
//!
//! Performs detailed comparison between TypeScript and Rust signal outputs,
//! classifying the result and calculating agreement scores.

use super::*;
use uuid::Uuid;

/// Engine for comparing signal outputs
pub struct ComparisonEngine {
    config: ParityConfig,
    comparison_counter: std::sync::atomic::AtomicU64,
}

impl ComparisonEngine {
    /// Create a new comparison engine
    pub fn new(config: &ParityConfig) -> Self {
        Self {
            config: config.clone(),
            comparison_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Compare two signal outputs and return a detailed comparison record
    pub fn compare(
        &self,
        ts_output: SignalOutput,
        rust_output: SignalOutput,
    ) -> Result<SignalComparisonRecord> {
        let counter = self
            .comparison_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let comparison_id = format!("cmp_{}_{}", Utc::now().timestamp_millis(), counter);

        let direction_match = ts_output.direction.agrees_with(&rust_output.direction);
        let confidence_diff = (ts_output.confidence - rust_output.confidence).abs();

        let entry_diff = Self::calculate_price_diff(ts_output.entry_price, rust_output.entry_price);
        let stop_diff = Self::calculate_price_diff(ts_output.stop_loss, rust_output.stop_loss);
        let target_diff = Self::calculate_price_diff(ts_output.take_profit, rust_output.take_profit);

        let pattern_comparisons = self.compare_patterns(&ts_output, &rust_output);
        let regime_comparison = self.compare_regimes(&ts_output, &rust_output);

        let comparison_type = self.classify_comparison(
            &ts_output,
            &rust_output,
            direction_match,
            confidence_diff,
            entry_diff,
        );

        let agreement_score = self.calculate_agreement_score(
            direction_match,
            confidence_diff,
            entry_diff,
            stop_diff,
            target_diff,
            &pattern_comparisons,
            &regime_comparison,
        );

        let notes = self.generate_notes(
            comparison_type,
            direction_match,
            confidence_diff,
            entry_diff,
        );

        Ok(SignalComparisonRecord {
            comparison_id,
            timestamp: Utc::now(),
            symbol: ts_output.symbol.clone(),
            timeframe: ts_output.timeframe.clone(),
            ts_output,
            rust_output,
            comparison_type,
            direction_match,
            confidence_diff,
            entry_diff,
            stop_diff,
            target_diff,
            pattern_comparisons,
            regime_comparison,
            agreement_score,
            notes,
        })
    }

    /// Calculate the difference between two optional prices
    fn calculate_price_diff(ts_price: Option<f64>, rust_price: Option<f64>) -> Option<f64> {
        match (ts_price, rust_price) {
            (Some(ts), Some(rust)) => Some((ts - rust).abs()),
            _ => None,
        }
    }

    /// Compare detected patterns between two outputs
    fn compare_patterns(&self, ts_output: &SignalOutput, rust_output: &SignalOutput) -> Vec<PatternComparison> {
        let rust_patterns: std::collections::HashSet<_> = rust_output.patterns.iter().cloned().collect();
        let ts_patterns: std::collections::HashSet<_> = ts_output.patterns.iter().cloned().collect();

        let all_patterns: std::collections::HashSet<_> = rust_patterns
            .union(&ts_patterns)
            .cloned()
            .collect();

        all_patterns
            .into_iter()
            .map(|pattern| {
                let rust_detected = rust_patterns.contains(&pattern);
                let ts_detected = ts_patterns.contains(&pattern);

                PatternComparison {
                    pattern_name: pattern,
                    rust_detected,
                    ts_detected,
                    rust_strength: if rust_detected { 1.0 } else { 0.0 },
                    ts_strength: if ts_detected { 1.0 } else { 0.0 },
                    agreement: rust_detected == ts_detected,
                }
            })
            .collect()
    }

    /// Compare regime classifications
    fn compare_regimes(&self, ts_output: &SignalOutput, rust_output: &SignalOutput) -> RegimeComparison {
        RegimeComparison {
            rust_regime: rust_output.regime.clone(),
            ts_regime: ts_output.regime.clone(),
            agreement: rust_output.regime == ts_output.regime,
        }
    }

    /// Classify the comparison type
    fn classify_comparison(
        &self,
        ts_output: &SignalOutput,
        rust_output: &SignalOutput,
        direction_match: bool,
        confidence_diff: f64,
        entry_diff: Option<f64>,
    ) -> ComparisonType {
        // Both have no signal
        if !ts_output.direction.is_actionable() && !rust_output.direction.is_actionable() {
            return ComparisonType::FalseNegative;
        }

        // Rust has signal, TypeScript doesn't
        if rust_output.direction.is_actionable() && !ts_output.direction.is_actionable() {
            return ComparisonType::FalsePositive;
        }

        // TypeScript has signal, Rust doesn't
        if ts_output.direction.is_actionable() && !rust_output.direction.is_actionable() {
            return ComparisonType::Miss;
        }

        // Both have signals - check agreement
        if direction_match {
            if confidence_diff < 1.0 && entry_diff.map_or(true, |d| d < self.config.price_tolerance_pips) {
                return ComparisonType::ExactMatch;
            }
            if confidence_diff < self.config.confidence_tolerance {
                return ComparisonType::CloseMatch;
            }
            return ComparisonType::PartialMatch;
        }

        ComparisonType::Disagreement
    }

    /// Calculate overall agreement score (0.0 - 1.0)
    fn calculate_agreement_score(
        &self,
        direction_match: bool,
        confidence_diff: f64,
        entry_diff: Option<f64>,
        stop_diff: Option<f64>,
        target_diff: Option<f64>,
        pattern_comparisons: &[PatternComparison],
        regime_comparison: &RegimeComparison,
    ) -> f64 {
        let mut score = 0.0;
        let mut weight = 0.0;

        // Direction match (40% weight)
        if direction_match {
            score += 0.4;
        }
        weight += 0.4;

        // Confidence similarity (20% weight)
        let confidence_score = (1.0 - (confidence_diff / 100.0)).max(0.0);
        score += confidence_score * 0.2;
        weight += 0.2;

        // Price level similarity (20% weight)
        let price_score = if let Some(entry) = entry_diff {
            let tolerance = self.config.price_tolerance_pips;
            let entry_score = (1.0 - (entry / tolerance)).max(0.0);
            let stop_score = stop_diff.map_or(1.0, |s| (1.0 - (s / tolerance)).max(0.0));
            let target_score = target_diff.map_or(1.0, |t| (1.0 - (t / tolerance)).max(0.0));
            (entry_score + stop_score + target_score) / 3.0
        } else {
            1.0 // Both have no prices, so they match
        };
        score += price_score * 0.2;
        weight += 0.2;

        // Pattern agreement (10% weight)
        if !pattern_comparisons.is_empty() {
            let pattern_agreements = pattern_comparisons.iter().filter(|p| p.agreement).count();
            let pattern_score = pattern_agreements as f64 / pattern_comparisons.len() as f64;
            score += pattern_score * 0.1;
            weight += 0.1;
        }

        // Regime agreement (10% weight)
        if regime_comparison.agreement {
            score += 0.1;
        }
        weight += 0.1;

        // Normalize to 0.0 - 1.0
        if weight > 0.0 {
            score / weight
        } else {
            0.0
        }
    }

    /// Generate human-readable notes for the comparison
    fn generate_notes(
        &self,
        comparison_type: ComparisonType,
        direction_match: bool,
        confidence_diff: f64,
        entry_diff: Option<f64>,
    ) -> String {
        let mut notes = Vec::new();

        match comparison_type {
            ComparisonType::ExactMatch => notes.push("Perfect match."),
            ComparisonType::CloseMatch => notes.push("Close match with minor differences."),
            ComparisonType::PartialMatch => notes.push("Partial agreement - review recommended."),
            ComparisonType::Disagreement => notes.push("DISAGREEMENT - requires investigation."),
            ComparisonType::Miss => notes.push("Rust missed TypeScript signal."),
            ComparisonType::FalsePositive => notes.push("Rust generated false positive."),
            ComparisonType::FalseNegative => notes.push("Both systems correctly no-signal."),
        }

        if !direction_match {
            notes.push("Direction mismatch.");
        }

        if confidence_diff > self.config.confidence_tolerance {
            notes.push(&format!("Confidence drift: {:.1}%", confidence_diff));
        }

        if let Some(diff) = entry_diff {
            if diff > self.config.price_tolerance_pips {
                notes.push(&format!("Entry price difference: {:.5}", diff));
            }
        }

        notes.join(" ")
    }
}

/// A stream of signal comparisons for batch processing
pub struct SignalComparisonStream {
    comparisons: Vec<SignalComparisonRecord>,
}

impl SignalComparisonStream {
    /// Create a new comparison stream
    pub fn new() -> Self {
        Self {
            comparisons: Vec::new(),
        }
    }

    /// Add a comparison to the stream
    pub fn add(&mut self, comparison: SignalComparisonRecord) {
        self.comparisons.push(comparison);
    }

    /// Get all comparisons
    pub fn comparisons(&self) -> &[SignalComparisonRecord] {
        &self.comparisons
    }

    /// Filter comparisons by type
    pub fn filter_by_type(&self, comp_type: ComparisonType) -> Vec<&SignalComparisonRecord> {
        self.comparisons
            .iter()
            .filter(|c| c.comparison_type == comp_type)
            .collect()
    }

    /// Get disagreement count
    pub fn disagreement_count(&self) -> usize {
        self.comparisons
            .iter()
            .filter(|c| matches!(c.comparison_type, ComparisonType::Disagreement))
            .count()
    }

    /// Get average agreement score
    pub fn average_agreement(&self) -> f64 {
        if self.comparisons.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.comparisons.iter().map(|c| c.agreement_score).sum();
        sum / self.comparisons.len() as f64
    }
}

impl Default for SignalComparisonStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_signal(direction: SignalDirection, confidence: f64) -> SignalOutput {
        SignalOutput {
            timestamp: Utc::now(),
            symbol: "EURUSD".to_string(),
            timeframe: "M15".to_string(),
            direction,
            confidence,
            confluence_score: 7.5,
            entry_price: Some(1.0850),
            stop_loss: Some(1.0820),
            take_profit: Some(1.0900),
            patterns: vec!["BOS".to_string(), "OB".to_string()],
            regime: "TRENDING".to_string(),
            session: "LONDON".to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_exact_match_classification() {
        let config = ParityConfig::default();
        let engine = ComparisonEngine::new(&config);

        let ts = create_test_signal(SignalDirection::Long, 75.0);
        let rust = create_test_signal(SignalDirection::Long, 75.5);

        let record = engine.compare(ts, rust).unwrap();
        assert_eq!(record.comparison_type, ComparisonType::ExactMatch);
        assert!(record.direction_match);
    }

    #[test]
    fn test_disagreement_classification() {
        let config = ParityConfig::default();
        let engine = ComparisonEngine::new(&config);

        let ts = create_test_signal(SignalDirection::Long, 75.0);
        let rust = create_test_signal(SignalDirection::Short, 75.0);

        let record = engine.compare(ts, rust).unwrap();
        assert_eq!(record.comparison_type, ComparisonType::Disagreement);
        assert!(!record.direction_match);
    }

    #[test]
    fn test_false_positive_classification() {
        let config = ParityConfig::default();
        let engine = ComparisonEngine::new(&config);

        let ts = create_test_signal(SignalDirection::NoSignal, 0.0);
        let rust = create_test_signal(SignalDirection::Long, 75.0);

        let record = engine.compare(ts, rust).unwrap();
        assert_eq!(record.comparison_type, ComparisonType::FalsePositive);
    }
}
