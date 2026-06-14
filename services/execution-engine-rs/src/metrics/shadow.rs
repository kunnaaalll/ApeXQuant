use metrics::{counter, histogram};

pub struct ShadowMetrics;

impl ShadowMetrics {
    pub fn record_parity_match(event_type: &str) {
        // Track when Rust implementation perfectly matches TypeScript implementation
        counter!("execution.shadow.parity.match", "event_type" => event_type.to_string()).increment(1);
    }

    pub fn record_parity_mismatch(event_type: &str, reason: &str) {
        // Track when Rust diverges from TypeScript implementation
        counter!("execution.shadow.parity.mismatch", "event_type" => event_type.to_string(), "reason" => reason.to_string()).increment(1);
    }
    
    pub fn record_latency_diff(diff_ms: f64) {
        // Positive means Rust is faster, Negative means TypeScript is faster
        histogram!("execution.shadow.latency_diff_ms").record(diff_ms);
    }
}
