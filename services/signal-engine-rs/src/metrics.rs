//! Prometheus metrics for the Signal Engine

use metrics::{counter, gauge, histogram, Unit};
use std::sync::atomic::{AtomicU64, Ordering};

/// Signal Engine metrics
#[derive(Debug, Default)]
pub struct SignalMetrics {
    /// Total signals generated
    signals_total: AtomicU64,
    /// Signals by quality grade
    signals_a_plus: AtomicU64,
    signals_a: AtomicU64,
    signals_b: AtomicU64,
    signals_rejected: AtomicU64,
    /// Processing latency in microseconds
    processing_latency_us: AtomicU64,
    /// Current active symbols being analyzed
    active_symbols: AtomicU64,
}

impl SignalMetrics {
    /// Get current metrics snapshot
    pub fn current() -> Self {
        Self::default()
    }

    /// Record a generated signal
    pub fn record_signal(&self, quality: &str) {
        counter!("apex_signal_generated_total", "quality" => quality.to_string()).increment(1);
        self.signals_total.fetch_add(1, Ordering::Relaxed);

        match quality {
            "A+" => self.signals_a_plus.fetch_add(1, Ordering::Relaxed),
            "A" => self.signals_a.fetch_add(1, Ordering::Relaxed),
            "B" => self.signals_b.fetch_add(1, Ordering::Relaxed),
            _ => self.signals_rejected.fetch_add(1, Ordering::Relaxed),
        };
    }

    /// Record processing latency
    pub fn record_latency(&self, latency_us: u64) {
        histogram!("apex_signal_processing_latency_us", Unit::Microseconds).record(latency_us);
        self.processing_latency_us.store(latency_us, Ordering::Relaxed);
    }

    /// Record active symbol count
    pub fn set_active_symbols(&self, count: u64) {
        gauge!("apex_signal_active_symbols").set(count as f64);
        self.active_symbols.store(count, Ordering::Relaxed);
    }

    /// Record pattern detection
    pub fn record_pattern_detected(pattern_type: &str, symbol: &str) {
        counter!(
            "apex_signal_pattern_detected_total",
            "pattern" => pattern_type.to_string(),
            "symbol" => symbol.to_string()
        )
        .increment(1);
    }

    /// Record confluence score distribution
    pub fn record_confluence_score(score: u8) {
        histogram!("apex_signal_confluence_score", Unit::Count).record(score as f64);
    }

    /// Record regime detection
    pub fn record_regime(regime_type: &str) {
        counter!("apex_signal_regime_detected_total", "regime" => regime_type.to_string())
            .increment(1);
    }

    /// Record MTF alignment score
    pub fn record_mtf_alignment(score: f64) {
        histogram!("apex_signal_mtf_alignment_score", Unit::Percent)
            .record(score * 100.0);
    }

    /// Get total signals count
    pub fn total_signals(&self) -> u64 {
        self.signals_total.load(Ordering::Relaxed)
    }
}

/// Initialize metrics registry
pub fn init_metrics() {
    metrics::describe_counter!(
        "apex_signal_generated_total",
        Unit::Count,
        "Total number of signals generated"
    );

    metrics::describe_histogram!(
        "apex_signal_processing_latency_us",
        Unit::Microseconds,
        "Signal processing latency in microseconds"
    );

    metrics::describe_gauge!(
        "apex_signal_active_symbols",
        Unit::Count,
        "Number of symbols currently being analyzed"
    );

    metrics::describe_histogram!(
        "apex_signal_confluence_score",
        Unit::Count,
        "Distribution of confluence scores"
    );
}
