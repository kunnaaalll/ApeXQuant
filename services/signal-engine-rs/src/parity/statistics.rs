//! Statistics Collector - Aggregate signal comparison metrics
//!
//! Tracks running totals, calculates agreement percentages, and maintains
//! historical statistics for trend analysis.

use super::*;
use std::collections::VecDeque;

/// Rolling window statistics
#[derive(Debug, Clone)]
pub struct RollingWindow {
    window_size: usize,
    values: VecDeque<f64>,
    sum: f64,
}

impl RollingWindow {
    /// Create a new rolling window
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            values: VecDeque::with_capacity(window_size),
            sum: 0.0,
        }
    }

    /// Add a value to the window
    pub fn add(&mut self, value: f64) {
        if self.values.len() >= self.window_size {
            if let Some(old) = self.values.pop_front() {
                self.sum -= old;
            }
        }
        self.values.push_back(value);
        self.sum += value;
    }

    /// Get the rolling average
    pub fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum / self.values.len() as f64
        }
    }

    /// Get the rolling standard deviation
    pub fn std_dev(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let avg = self.average();
        let variance: f64 =
            self.values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / self.values.len() as f64;
        variance.sqrt()
    }

    /// Get the count of values
    pub fn count(&self) -> usize {
        self.values.len()
    }
}

/// Comprehensive statistics for signal comparisons
#[derive(Debug, Clone)]
pub struct ParityStatistics {
    pub total_comparisons: u64,
    pub by_type: HashMap<ComparisonType, u64>,
    pub direction_agreement_count: u64,
    pub avg_confidence_diff: RollingWindow,
    pub avg_entry_diff: RollingWindow,
    pub avg_stop_diff: RollingWindow,
    pub avg_target_diff: RollingWindow,
    pub agreement_scores: RollingWindow,
    pub by_symbol: HashMap<String, SymbolStatistics>,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

impl ParityStatistics {
    /// Create new statistics collector
    pub fn new(window_size: usize) -> Self {
        let mut by_type = HashMap::new();
        for ct in [
            ComparisonType::ExactMatch,
            ComparisonType::CloseMatch,
            ComparisonType::PartialMatch,
            ComparisonType::Disagreement,
            ComparisonType::Miss,
            ComparisonType::FalsePositive,
            ComparisonType::FalseNegative,
        ] {
            by_type.insert(ct, 0);
        }

        Self {
            total_comparisons: 0,
            by_type,
            direction_agreement_count: 0,
            avg_confidence_diff: RollingWindow::new(window_size),
            avg_entry_diff: RollingWindow::new(window_size),
            avg_stop_diff: RollingWindow::new(window_size),
            avg_target_diff: RollingWindow::new(window_size),
            agreement_scores: RollingWindow::new(window_size),
            by_symbol: HashMap::new(),
            start_time: Utc::now(),
            last_update: Utc::now(),
        }
    }

    /// Record a comparison
    pub fn record(&mut self, comparison: &SignalComparisonRecord) {
        self.total_comparisons += 1;

        // Count by type
        if let Some(count) = self.by_type.get_mut(&comparison.comparison_type) {
            *count += 1;
        }

        // Count direction agreement
        if comparison.direction_match {
            self.direction_agreement_count += 1;
        }

        // Update rolling averages
        self.avg_confidence_diff.add(comparison.confidence_diff);
        self.agreement_scores.add(comparison.agreement_score);

        if let Some(diff) = comparison.entry_diff {
            self.avg_entry_diff.add(diff);
        }
        if let Some(diff) = comparison.stop_diff {
            self.avg_stop_diff.add(diff);
        }
        if let Some(diff) = comparison.target_diff {
            self.avg_target_diff.add(diff);
        }

        // Update symbol statistics
        self.by_symbol
            .entry(comparison.symbol.clone())
            .or_insert_with(|| SymbolStatistics::new())
            .record(comparison);

        self.last_update = Utc::now();
    }

    /// Get direction agreement percentage
    pub fn direction_agreement_pct(&self) -> f64 {
        if self.total_comparisons == 0 {
            0.0
        } else {
            (self.direction_agreement_count as f64 / self.total_comparisons as f64) * 100.0
        }
    }

    /// Get comparison type breakdown
    pub fn type_breakdown(&self) -> HashMap<ComparisonType, (u64, f64)> {
        self.by_type
            .iter()
            .map(|(ct, count)| {
                let pct = if self.total_comparisons > 0 {
                    (*count as f64 / self.total_comparisons as f64) * 100.0
                } else {
                    0.0
                };
                (*ct, (*count, pct))
            })
            .collect()
    }

    /// Generate summary metrics
    pub fn to_metrics(&self) -> AgreementMetrics {
        let mut metrics = AgreementMetrics {
            total_comparisons: self.total_comparisons,
            exact_matches: *self.by_type.get(&ComparisonType::ExactMatch).unwrap_or(&0),
            close_matches: *self.by_type.get(&ComparisonType::CloseMatch).unwrap_or(&0),
            partial_matches: *self
                .by_type
                .get(&ComparisonType::PartialMatch)
                .unwrap_or(&0),
            disagreements: *self
                .by_type
                .get(&ComparisonType::Disagreement)
                .unwrap_or(&0),
            misses: *self.by_type.get(&ComparisonType::Miss).unwrap_or(&0),
            false_positives: *self
                .by_type
                .get(&ComparisonType::FalsePositive)
                .unwrap_or(&0),
            false_negatives: *self
                .by_type
                .get(&ComparisonType::FalseNegative)
                .unwrap_or(&0),
            direction_agreement_pct: self.direction_agreement_pct(),
            avg_confidence_diff: self.avg_confidence_diff.average(),
            avg_entry_diff: self.avg_entry_diff.average(),
            avg_stop_diff: self.avg_stop_diff.average(),
            avg_target_diff: self.avg_target_diff.average(),
        };

        metrics.calculate_direction_agreement();
        metrics
    }

    /// Get runtime duration
    pub fn runtime_duration(&self) -> chrono::Duration {
        self.last_update.signed_duration_since(self.start_time)
    }

    /// Get comparison rate (comparisons per hour)
    pub fn comparison_rate(&self) -> f64 {
        let hours = self.runtime_duration().num_seconds() as f64 / 3600.0;
        if hours > 0.0 {
            self.total_comparisons as f64 / hours
        } else {
            0.0
        }
    }
}

impl Default for ParityStatistics {
    fn default() -> Self {
        Self::new(1000) // Default 1000-item rolling window
    }
}

/// Statistics per symbol
#[derive(Debug, Clone)]
pub struct SymbolStatistics {
    pub comparison_count: u64,
    pub direction_agreements: u64,
    pub exact_matches: u64,
    pub disagreements: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
    pub avg_agreement_score: f64,
}

impl SymbolStatistics {
    /// Create new symbol statistics
    pub fn new() -> Self {
        Self {
            comparison_count: 0,
            direction_agreements: 0,
            exact_matches: 0,
            disagreements: 0,
            false_positives: 0,
            false_negatives: 0,
            avg_agreement_score: 0.0,
        }
    }

    /// Record a comparison for this symbol
    pub fn record(&mut self, comparison: &SignalComparisonRecord) {
        self.comparison_count += 1;

        if comparison.direction_match {
            self.direction_agreements += 1;
        }

        match comparison.comparison_type {
            ComparisonType::ExactMatch => self.exact_matches += 1,
            ComparisonType::Disagreement => self.disagreements += 1,
            ComparisonType::FalsePositive => self.false_positives += 1,
            ComparisonType::FalseNegative => self.false_negatives += 1,
            _ => {}
        }

        // Update running average of agreement score
        self.avg_agreement_score = ((self.avg_agreement_score
            * (self.comparison_count - 1) as f64)
            + comparison.agreement_score)
            / self.comparison_count as f64;
    }

    /// Get direction agreement percentage for this symbol
    pub fn direction_agreement_pct(&self) -> f64 {
        if self.comparison_count == 0 {
            0.0
        } else {
            (self.direction_agreements as f64 / self.comparison_count as f64) * 100.0
        }
    }
}

impl Default for SymbolStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe statistics collector
pub struct StatisticsCollector {
    inner: std::sync::Mutex<ParityStatistics>,
}

impl StatisticsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(ParityStatistics::default()),
        }
    }

    /// Record a comparison
    pub fn record(&self, comparison: &SignalComparisonRecord) -> Result<()> {
        let mut stats = self
            .inner
            .lock()
            .map_err(|_| ParityError::StatisticsError("Failed to lock statistics".to_string()))?;
        stats.record(comparison);
        Ok(())
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> AgreementMetrics {
        let stats = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        stats.to_metrics()
    }

    /// Get full statistics
    pub fn get_statistics(&self) -> ParityStatistics {
        let stats = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        stats.clone()
    }

    /// Reset statistics
    pub fn reset(&self) -> Result<()> {
        let mut stats = self
            .inner
            .lock()
            .map_err(|_| ParityError::StatisticsError("Failed to lock statistics".to_string()))?;
        *stats = ParityStatistics::default();
        Ok(())
    }
}

impl Default for StatisticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_window() {
        let mut window = RollingWindow::new(3);
        window.add(1.0);
        window.add(2.0);
        window.add(3.0);
        assert_eq!(window.average(), 2.0);

        window.add(4.0);
        assert_eq!(window.average(), 3.0); // (2+3+4)/3
    }

    #[test]
    fn test_parity_statistics() {
        let mut stats = ParityStatistics::new(100);
        assert_eq!(stats.direction_agreement_pct(), 0.0);

        let record = SignalComparisonRecord {
            comparison_id: "test".to_string(),
            timestamp: Utc::now(),
            symbol: "EURUSD".to_string(),
            timeframe: "M15".to_string(),
            ts_output: SignalOutput {
                timestamp: Utc::now(),
                symbol: "EURUSD".to_string(),
                timeframe: "M15".to_string(),
                direction: SignalDirection::Long,
                confidence: 75.0,
                confluence_score: 7.5,
                entry_price: Some(1.0850),
                stop_loss: Some(1.0820),
                take_profit: Some(1.0900),
                patterns: vec![],
                regime: "TRENDING".to_string(),
                session: "LONDON".to_string(),
                metadata: HashMap::new(),
            },
            rust_output: SignalOutput {
                timestamp: Utc::now(),
                symbol: "EURUSD".to_string(),
                timeframe: "M15".to_string(),
                direction: SignalDirection::Long,
                confidence: 76.0,
                confluence_score: 7.5,
                entry_price: Some(1.0851),
                stop_loss: Some(1.0821),
                take_profit: Some(1.0901),
                patterns: vec![],
                regime: "TRENDING".to_string(),
                session: "LONDON".to_string(),
                metadata: HashMap::new(),
            },
            comparison_type: ComparisonType::CloseMatch,
            direction_match: true,
            confidence_diff: 1.0,
            entry_diff: Some(0.0001),
            stop_diff: Some(0.0001),
            target_diff: Some(0.0001),
            pattern_comparisons: vec![],
            regime_comparison: RegimeComparison {
                rust_regime: "TRENDING".to_string(),
                ts_regime: "TRENDING".to_string(),
                agreement: true,
            },
            agreement_score: 0.95,
            notes: "Test comparison".to_string(),
        };

        stats.record(&record);
        assert_eq!(stats.total_comparisons, 1);
        assert_eq!(stats.direction_agreement_pct(), 100.0);
    }
}
